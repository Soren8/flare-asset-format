use std::collections::HashMap;

use crate::error::{FlareError, Result};
use crate::parser::{parse_file_with_resolver, parse_str, Entry, FileResolver};
use crate::value::{parse_duration_ms, parse_int, parse_point, pop_first_token, Point, Rect};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileAnimFrame {
    pub x: i32,
    pub y: i32,
    pub duration_ms: u32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileAnim {
    pub frames: Vec<TileAnimFrame>,
}

#[derive(Debug, Clone)]
struct TileAnimState {
    anim: TileAnim,
    current_frame: usize,
    elapsed_ms: f32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub atlas_index: usize,
    pub src: Rect,
    pub offset: Point,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileAtlas {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct TileSet {
    pub atlases: Vec<TileAtlas>,
    pub tiles: HashMap<u32, Tile>,
    pub animations: HashMap<u32, TileAnim>,
    anim_states: HashMap<u32, TileAnimState>,
}

impl TileSet {
    pub fn parse_str(content: &str) -> Result<Self> {
        Self::from_entries(&parse_str(content, "tileset.txt")?)
    }

    pub fn parse_file(path: &str, resolver: &dyn FileResolver) -> Result<Self> {
        Self::from_entries(&parse_file_with_resolver(path, resolver)?)
    }

    pub fn from_entries(entries: &[Entry]) -> Result<Self> {
        let mut set = Self {
            atlases: Vec::new(),
            tiles: HashMap::new(),
            animations: HashMap::new(),
            anim_states: HashMap::new(),
        };

        let mut current_atlas = 0usize;

        for entry in entries {
            if entry.new_section && entry.section == "tileset" {
                set.atlases.push(TileAtlas {
                    path: String::new(),
                });
                current_atlas = set.atlases.len().saturating_sub(1);
                continue;
            }

            if set.atlases.is_empty() {
                set.atlases.push(TileAtlas {
                    path: String::new(),
                });
            }

            match entry.key.as_str() {
                "img" => {
                    set.atlases[current_atlas].path = entry.value.clone();
                }
                "tile" => {
                    let mut rest = entry.value.as_str();
                    let index = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
                        FlareError::Other("tile line missing index".into())
                    })?)? as u32;
                    if index == 0 {
                        continue;
                    }
                    let x = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
                        FlareError::Other("tile line missing x".into())
                    })?)?;
                    let y = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
                        FlareError::Other("tile line missing y".into())
                    })?)?;
                    let w = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
                        FlareError::Other("tile line missing width".into())
                    })?)?;
                    let h = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
                        FlareError::Other("tile line missing height".into())
                    })?)?;
                    let offset = parse_point(rest)?;
                    set.tiles.insert(
                        index,
                        Tile {
                            atlas_index: current_atlas,
                            src: Rect { x, y, w, h },
                            offset,
                        },
                    );
                }
                "animation" => {
                    // flare-game exports sometimes use ';' between frame tuples while tiles use ','.
                    let normalized = entry.value.replace(';', ",");
                    let mut rest = normalized.as_str();
                    let index = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
                        FlareError::Other("animation line missing index".into())
                    })?)? as u32;
                    let mut frames = Vec::new();
                    while let Some(x_str) = pop_first_token(&mut rest) {
                        let y = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
                            FlareError::Other("animation frame missing y".into())
                        })?)?;
                        let duration_ms = parse_duration_ms(&pop_first_token(&mut rest).ok_or_else(
                            || FlareError::Other("animation frame missing duration".into()),
                        )?)?;
                        frames.push(TileAnimFrame {
                            x: parse_int(&x_str)?,
                            y,
                            duration_ms,
                        });
                    }
                    set.animations.insert(index, TileAnim { frames });
                }
                _ => {}
            }
        }

        Ok(set)
    }

    pub fn tile(&self, id: u32) -> Option<&Tile> {
        self.tiles.get(&id)
    }

    pub fn atlas_path(&self, atlas_index: usize) -> Option<&str> {
        self.atlases.get(atlas_index).map(|a| a.path.as_str())
    }

    pub fn current_src(&self, id: u32) -> Option<Rect> {
        let tile = self.tiles.get(&id)?;
        if let Some(state) = self.anim_states.get(&id) {
            if let Some(frame) = state.anim.frames.get(state.current_frame) {
                return Some(Rect {
                    x: frame.x,
                    y: frame.y,
                    w: tile.src.w,
                    h: tile.src.h,
                });
            }
        }
        Some(tile.src)
    }

    pub fn advance(&mut self, dt_ms: f32) {
        for (id, anim) in &self.animations {
            let state = self
                .anim_states
                .entry(*id)
                .or_insert_with(|| TileAnimState {
                    anim: anim.clone(),
                    current_frame: 0,
                    elapsed_ms: 0.0,
                });

            if state.anim.frames.is_empty() {
                continue;
            }

            state.elapsed_ms += dt_ms;
            let frame_duration = state.anim.frames[state.current_frame].duration_ms as f32;
            if state.elapsed_ms >= frame_duration {
                state.elapsed_ms = 0.0;
                state.current_frame = (state.current_frame + 1) % state.anim.frames.len();
            }
        }
    }
}
