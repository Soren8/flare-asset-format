# flare-format

Engine-agnostic Rust library for loading the FLARE art format (see [format spec](../../docs/flare-art-format-spec.md)).

Parses FLARE `.txt` definition files for **animations**, **tilesets**, **maps**, **parallax backgrounds**, and **icons**. Provides runtime helpers for animation playback, tile animation cycling, map collision walkability, and tile/screen coordinate conversion.

This crate does **not** load PNG files, decode images, or render anything. It returns image paths, source rectangles, and offsets for your engine to consume.

## Features

- INI-style FLARE config parsing (`APPEND`, `INCLUDE`, mod overlay resolution)
- Animation definitions (compressed per-frame and uncompressed grid layouts)
- Tileset definitions with animated tiles
- Map layers and collision data
- Millisecond-based animation playback (decoupled from FLARE engine FPS)
- Isometric and orthogonal coordinate helpers
- Optional `serde` feature for serialization

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
flare-format = { path = "rust/flare-format" }
```

Load assets from a mod directory:

```rust
use std::path::PathBuf;
use flare_format::{AnimationSet, CollisionMap, Direction, FsResolver, Map, TileSet};

let resolver = FsResolver::new([PathBuf::from("mods/fantasycore")]);

let anim = AnimationSet::parse_file("animations/loot/coins5.txt", &resolver)?;
let tileset = TileSet::parse_file("tilesetdefs/tileset_grassland.txt", &resolver)?;
let map = Map::parse_file("maps/arrival.txt", &resolver)?;
let collision = CollisionMap::from_map(&map);

assert!(!collision.is_wall(0, 0));
```

## Macroquad integration

`flare-format` has no Macroquad dependency. Load textures and draw sub-rectangles yourself:

```rust
// Pseudocode — in your Macroquad game loop:
let texture = load_texture(format!("{mod_root}/{}", image_path)).await?;
let frame = player.current_frame(Direction::Sw).unwrap();

draw_texture_ex(
    &texture,
    screen_x - frame.offset.x as f32,
    screen_y - frame.offset.y as f32,
    WHITE,
    DrawTextureParams {
        source: Some(macroquad::math::Rect::new(
            frame.src.x as f32,
            frame.src.y as f32,
            frame.src.w as f32,
            frame.src.h as f32,
        )),
        ..Default::default()
    },
);
```

## Modules

| Module | Purpose |
|--------|---------|
| `animation` | Sprite animation definitions and `AnimationPlayer` |
| `tileset` | Tileset definitions and tile animation cycling |
| `map` | Map header, layers, and passthrough entity/event sections |
| `collision` | Collision layer walkability queries |
| `config` | `engine/tileset_config.txt` parsing |
| `coords` | Map ↔ screen coordinate conversion |
| `parallax` | Parallax background layer definitions |
| `icons` | Icon atlas registration |
| `parser` | Low-level INI parser and `FsResolver` |

## Format reference

See [docs/flare-art-format-spec.md](../../docs/flare-art-format-spec.md) in this repository.

## License

CC-BY-SA 3.0 — same as the art assets in this repository. See [LICENSE.txt](../../LICENSE.txt).
