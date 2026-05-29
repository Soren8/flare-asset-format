use crate::map::{Map, MapLayer};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u16)]
pub enum CollisionType {
    #[default]
    BlocksNone = 0,
    BlocksAll = 1,
    BlocksMovement = 2,
    BlocksAllHidden = 3,
    BlocksMovementHidden = 4,
    MapOnly = 5,
    MapOnlyAlt = 6,
    BlocksEntities = 7,
    BlocksEnemies = 8,
}

impl CollisionType {
    pub fn from_u16(value: u16) -> Self {
        match value {
            1 => Self::BlocksAll,
            2 => Self::BlocksMovement,
            3 => Self::BlocksAllHidden,
            4 => Self::BlocksMovementHidden,
            5 => Self::MapOnly,
            6 => Self::MapOnlyAlt,
            7 => Self::BlocksEntities,
            8 => Self::BlocksEnemies,
            _ => Self::BlocksNone,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MovementType {
    #[default]
    Normal,
    Flying,
    Intangible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionMap {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<Vec<CollisionType>>,
}

impl CollisionMap {
    pub fn from_layer(layer: &MapLayer) -> Self {
        let height = layer.data.len() as u16;
        let width = layer
            .data
            .first()
            .map(|row| row.len() as u16)
            .unwrap_or(0);
        let mut tiles = vec![vec![CollisionType::BlocksNone; height as usize]; width as usize];
        for (y, row) in layer.data.iter().enumerate() {
            for (x, value) in row.iter().enumerate() {
                tiles[x][y] = CollisionType::from_u16(*value);
            }
        }
        Self {
            width,
            height,
            tiles,
        }
    }

    pub fn from_map(map: &Map) -> Self {
        if let Some(layer) = map.collision_layer() {
            Self::from_layer(layer)
        } else {
            Self {
                width: map.header.width,
                height: map.header.height,
                tiles: vec![
                    vec![CollisionType::BlocksNone; map.header.height as usize];
                    map.header.width as usize
                ],
            }
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u16) < self.width && (y as u16) < self.height
    }

    pub fn get(&self, x: i32, y: i32) -> Option<CollisionType> {
        if !self.in_bounds(x, y) {
            return None;
        }
        Some(self.tiles[x as usize][y as usize])
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        matches!(
            self.get(x, y),
            Some(CollisionType::BlocksAll | CollisionType::BlocksAllHidden)
        )
    }

    pub fn blocks_movement(&self, x: i32, y: i32, movement: MovementType) -> bool {
        if !self.in_bounds(x, y) {
            return true;
        }

        let tile = self.tiles[x as usize][y as usize];
        match movement {
            MovementType::Intangible => false,
            MovementType::Flying => {
                matches!(tile, CollisionType::BlocksAll | CollisionType::BlocksAllHidden)
            }
            MovementType::Normal => !matches!(
                tile,
                CollisionType::BlocksNone | CollisionType::MapOnly | CollisionType::MapOnlyAlt
            ),
        }
    }

    pub fn can_occupy(&self, x: i32, y: i32, movement: MovementType) -> bool {
        !self.blocks_movement(x, y, movement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_and_walkable() {
        let layer = MapLayer {
            name: "collision".into(),
            data: vec![
                vec![0, 1],
                vec![2, 0],
            ],
        };
        let map = CollisionMap::from_layer(&layer);
        assert!(!map.is_wall(0, 0));
        assert!(map.is_wall(1, 0));
        assert!(map.blocks_movement(1, 0, MovementType::Flying));
        assert!(map.blocks_movement(1, 0, MovementType::Normal));
        assert!(map.blocks_movement(0, 1, MovementType::Normal));
        assert!(!map.blocks_movement(0, 1, MovementType::Flying));
        assert!(!map.blocks_movement(1, 1, MovementType::Normal));
        assert!(!map.blocks_movement(0, 1, MovementType::Intangible));
    }
}
