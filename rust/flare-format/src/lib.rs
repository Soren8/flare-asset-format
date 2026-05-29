//! # flare-format
//!
//! Engine-agnostic parser and runtime helpers for the FLARE art format.
//!
//! This crate loads FLARE `.txt` definition files for animations, tilesets, maps,
//! parallax backgrounds, and icons. It returns image **paths**, **source rectangles**,
//! and **offsets** — no rendering or texture loading is performed.
//!
//! ## Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use flare_format::{AnimationSet, FsResolver, TileSet};
//!
//! let resolver = FsResolver::new([PathBuf::from("/path/to/mods/fantasycore")]);
//!
//! let anim = AnimationSet::parse_file("animations/loot/coins5.txt", &resolver)?;
//! let tileset = TileSet::parse_file("tilesetdefs/tileset_grassland.txt", &resolver)?;
//!
//! if let Some(frame) = anim.default_animation().and_then(|a| a.frame(0, flare_format::Direction::Sw)) {
//!     println!("image: {:?}", anim.image_path(&frame.image_id));
//!     println!("src: {:?}", frame.src);
//! }
//! # Ok::<(), flare_format::FlareError>(())
//! ```
//!
//! ## Macroquad integration
//!
//! Load textures in your game engine and draw sub-rectangles from parsed definitions:
//!
//! ```ignore
//! // Pseudocode — Macroquad is not a dependency of this crate.
//! let texture = load_texture(mod_root.join(tile_path)).await?;
//! let src = frame.src;
//! draw_texture_ex(
//!     &texture,
//!     dest_x - frame.offset.x as f32,
//!     dest_y - frame.offset.y as f32,
//!     WHITE,
//!     DrawTextureParams {
//!         source: Some(Rect::new(src.x as f32, src.y as f32, src.w as f32, src.h as f32)),
//!         ..Default::default()
//!     },
//! );
//! ```
//!
//! ## Durations
//!
//! Animation and tile-animation durations are stored in **milliseconds**. Playback
//! helpers advance by real delta time. This differs from the C++ engine, which converts
//! durations to logic frames at `max_frames_per_sec`.

pub mod animation;
pub mod collision;
pub mod config;
pub mod coords;
pub mod error;
pub mod icons;
pub mod map;
pub mod parser;
pub mod parallax;
pub mod tileset;
pub mod value;

pub use animation::{
    Animation, AnimationEvent, AnimationPlayer, AnimationSet, Frame, DIRECTIONS,
};
pub use collision::{CollisionMap, CollisionType, MovementType};
pub use config::TilesetConfig;
pub use coords::{map_to_screen, screen_to_map};
pub use error::{FlareError, Result};
pub use icons::{IconConfig, IconSet};
pub use map::{Map, MapHeader, MapLayer, RawSection, TilesetExportEntry};
pub use parser::{parse_file_with_resolver, parse_str, Entry, FileResolver, FsResolver};
pub use parallax::{ParallaxLayer, ParallaxLayers};
pub use tileset::{Tile, TileAnim, TileAnimFrame, TileAtlas, TileSet};
pub use value::{
    ActiveSubFrame, AnimationType, BlendMode, Color, Direction, Orientation, Point, Rect,
};
