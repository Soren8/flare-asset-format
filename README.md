# flare-asset-format

Standalone repository for the **FLARE art asset format**: reference assets, format specification, and an engine-agnostic Rust loader library.

This repo is intended for game engines and tools that want to import FLARE sprites, tilesets, and maps without depending on the FLARE C++ engine.

## Contents

| Path | Description |
|------|-------------|
| [`docs/flare-art-format-spec.md`](docs/flare-art-format-spec.md) | Normative format specification |
| [`rust/flare-format/`](rust/flare-format/) | Rust parser/loader crate (no rendering dependencies) |
| [`mods/`](mods/) | Runtime game data (animations, images, tilesets, maps) |
| [`art_src/`](art_src/) | Authoring source art (Blender, raw textures, animation templates) |
| [`tiled/`](tiled/) | Tiled map authoring files (`.tmx`, source tile PNGs, autotile rules) |
| [`LICENSE.txt`](LICENSE.txt) | CC-BY-SA 3.0 license text |
| [`CREDITS.txt`](CREDITS.txt) | Artist and contributor attribution |

## License

All art and data files in this repository are released under **Creative Commons Attribution-ShareAlike 3.0** (CC-BY-SA 3.0). See [`LICENSE.txt`](LICENSE.txt).

You must preserve attribution when redistributing or adapting this work. See [`CREDITS.txt`](CREDITS.txt) and per-folder credit files under `art_src/` and `tiled/` (e.g. `art_src/tilesets/grassland/CREDITS.txt`).

Per-file attribution is also maintained on the upstream wiki:  
https://github.com/flareteam/flare-game/wiki/Credits

## Provenance

Assets and format documentation are derived from the [flare-game](https://github.com/flareteam/flare-game) and [flare-engine](https://github.com/flareteam/flare-engine) projects. This repository contains **no FLARE engine source code**.

Font files (SIL Open Font License) and engine GPL code are intentionally excluded.

## Using the Rust loader

```toml
[dependencies]
flare-format = { path = "rust/flare-format" }
```

```rust
use std::path::PathBuf;
use flare_format::{AnimationSet, FsResolver, Map, TileSet};

let resolver = FsResolver::new([PathBuf::from("mods/fantasycore")]);
let anim = AnimationSet::parse_file("animations/enemies/zombie.txt", &resolver)?;
let tileset = TileSet::parse_file("tilesetdefs/tileset_grassland.txt", &resolver)?;
let map = Map::parse_file("../empyrean_campaign/maps/arrival.txt", &resolver)?;
```

See [`rust/flare-format/README.md`](rust/flare-format/README.md) for details.

## Mod layout

Runtime assets follow the FLARE mod structure:

```
mods/
├── fantasycore/       # Base art pack (sprites, tilesets, icons)
├── empyrean_campaign/ # Campaign maps and additional art
├── alpha_demo/        # Demo content
├── minicore/          # Half-scale art variant
└── ...
```

Authoring workflow:

- Edit maps in **Tiled** using files under `tiled/`
- Export/bake runtime assets into `mods/` (animations + images + tilesetdefs + maps)
- Keep Blender/source files in `art_src/` for regeneration

## Building the loader

```bash
cd rust/flare-format
cargo test
cargo clippy -- -D warnings
```
