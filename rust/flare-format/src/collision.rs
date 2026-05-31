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

    /// Whether a float map position is inside the map and walkable.
    pub fn is_valid_position(&self, x: f32, y: f32, movement: MovementType) -> bool {
        if x < 0.0 || y < 0.0 {
            return false;
        }
        let tile_x = x as i32;
        let tile_y = y as i32;
        if !self.in_bounds(tile_x, tile_y) {
            return false;
        }
        self.can_occupy(tile_x, tile_y, movement)
    }

    /// Port of `MapCollision::move()` with tile-boundary sub-steps and wall sliding.
    pub fn move_position(
        &self,
        mut x: f32,
        mut y: f32,
        step_x: f32,
        step_y: f32,
        movement: MovementType,
    ) -> (f32, f32, bool) {
        const MIN_TILE_GAP: f32 = 0.001;

        let mut remaining_x = step_x;
        let mut remaining_y = step_y;
        let force_slide = step_x != 0.0 && step_y != 0.0;

        while remaining_x != 0.0 || remaining_y != 0.0 {
            let mut sub_x = 0.0;
            if remaining_x > 0.0 {
                sub_x = (x.ceil() - x).min(remaining_x);
                if sub_x <= MIN_TILE_GAP {
                    sub_x = 1.0_f32.min(remaining_x);
                }
            } else if remaining_x < 0.0 {
                sub_x = (x.floor() - x).max(remaining_x);
                if sub_x == 0.0 {
                    sub_x = (-1.0_f32).max(remaining_x);
                }
            }

            let mut sub_y = 0.0;
            if remaining_y > 0.0 {
                sub_y = (y.ceil() - y).min(remaining_y);
                if sub_y <= MIN_TILE_GAP {
                    sub_y = 1.0_f32.min(remaining_y);
                }
            } else if remaining_y < 0.0 {
                sub_y = (y.floor() - y).max(remaining_y);
                if sub_y == 0.0 {
                    sub_y = (-1.0_f32).max(remaining_y);
                }
            }

            remaining_x -= sub_x;
            remaining_y -= sub_y;

            if !Self::small_step(self, &mut x, &mut y, sub_x, sub_y, movement) {
                if force_slide {
                    if !Self::small_step_forced_slide_along_grid(
                        self, &mut x, &mut y, sub_x, sub_y, movement,
                    ) {
                        return (x, y, false);
                    }
                } else if !Self::small_step_forced_slide(
                    self, &mut x, &mut y, sub_x, sub_y, movement,
                ) {
                    return (x, y, false);
                }
            }
        }

        (x, y, true)
    }

    fn small_step(
        &self,
        x: &mut f32,
        y: &mut f32,
        step_x: f32,
        step_y: f32,
        movement: MovementType,
    ) -> bool {
        if self.is_valid_position(*x + step_x, *y + step_y, movement) {
            *x += step_x;
            *y += step_y;
            true
        } else {
            false
        }
    }

    fn small_step_forced_slide_along_grid(
        &self,
        x: &mut f32,
        y: &mut f32,
        step_x: f32,
        step_y: f32,
        movement: MovementType,
    ) -> bool {
        if self.is_valid_position(*x + step_x, *y, movement) {
            if step_x == 0.0 {
                return true;
            }
            *x += step_x;
        } else if self.is_valid_position(*x, *y + step_y, movement) {
            if step_y == 0.0 {
                return true;
            }
            *y += step_y;
        } else {
            return false;
        }
        true
    }

    fn small_step_forced_slide(
        &self,
        x: &mut f32,
        y: &mut f32,
        step_x: f32,
        step_y: f32,
        movement: MovementType,
    ) -> bool {
        const EPSILON: f32 = 0.01;

        if step_x != 0.0 {
            debug_assert_eq!(step_y, 0.0);
            let dy = *y - y.floor();
            let tile_x = *x as i32;
            let tile_y = *y as i32;
            if self.can_occupy(tile_x, tile_y + 1, movement)
                && self.can_occupy(tile_x + step_x.signum() as i32, tile_y + 1, movement)
                && dy > 0.5
            {
                *y += (1.0 - dy + EPSILON).min(step_x.abs());
            } else if self.can_occupy(tile_x, tile_y - 1, movement)
                && self.can_occupy(tile_x + step_x.signum() as i32, tile_y - 1, movement)
                && dy < 0.5
            {
                *y -= (dy + EPSILON).min(step_x.abs());
            } else {
                return false;
            }
        } else if step_y != 0.0 {
            debug_assert_eq!(step_x, 0.0);
            let dx = *x - x.floor();
            let tile_x = *x as i32;
            let tile_y = *y as i32;
            if self.can_occupy(tile_x + 1, tile_y, movement)
                && self.can_occupy(tile_x + 1, tile_y + step_y.signum() as i32, movement)
                && dx > 0.5
            {
                *x += (1.0 - dx + EPSILON).min(step_y.abs());
            } else if self.can_occupy(tile_x - 1, tile_y, movement)
                && self.can_occupy(tile_x - 1, tile_y + step_y.signum() as i32, movement)
                && dx < 0.5
            {
                *x -= (dx + EPSILON).min(step_y.abs());
            } else {
                return false;
            }
        } else {
            return false;
        }
        true
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
