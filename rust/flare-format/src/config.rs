use crate::error::{FlareError, Result};
use crate::parser::{parse_file_with_resolver, parse_str, Entry, FileResolver};
use crate::value::{parse_orientation, parse_point, Orientation};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TilesetConfig {
    pub orientation: Orientation,
    pub tile_width: i32,
    pub tile_height: i32,
}

impl Default for TilesetConfig {
    fn default() -> Self {
        Self {
            orientation: Orientation::Isometric,
            tile_width: 64,
            tile_height: 32,
        }
    }
}

impl TilesetConfig {
    pub fn from_entries(entries: &[Entry]) -> Result<Self> {
        let mut config = Self::default();
        for entry in entries {
            if !entry.section.is_empty() {
                continue;
            }
            match entry.key.as_str() {
                "orientation" => config.orientation = parse_orientation(&entry.value)?,
                "tile_size" => {
                    let size = parse_point(&entry.value)?;
                    config.tile_width = size.x;
                    config.tile_height = size.y;
                }
                _ => {}
            }
        }
        if config.tile_width <= 0 || config.tile_height <= 0 {
            return Err(FlareError::Other(
                "tile_size width and height must be greater than 0".into(),
            ));
        }
        Ok(config)
    }

    pub fn parse_str(content: &str) -> Result<Self> {
        Self::from_entries(&parse_str(content, "engine/tileset_config.txt")?)
    }

    pub fn parse_file(path: &str, resolver: &dyn FileResolver) -> Result<Self> {
        Self::from_entries(&parse_file_with_resolver(path, resolver)?)
    }

    pub fn tile_width_half(&self) -> i32 {
        self.tile_width / 2
    }

    pub fn tile_height_half(&self) -> i32 {
        self.tile_height / 2
    }

    pub fn units_per_pixel_x(&self) -> f32 {
        match self.orientation {
            Orientation::Isometric => 2.0 / self.tile_width as f32,
            Orientation::Orthogonal => 1.0 / self.tile_width as f32,
        }
    }

    pub fn units_per_pixel_y(&self) -> f32 {
        match self.orientation {
            Orientation::Isometric => 2.0 / self.tile_height as f32,
            Orientation::Orthogonal => 1.0 / self.tile_height as f32,
        }
    }
}
