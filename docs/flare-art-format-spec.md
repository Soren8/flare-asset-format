# FLARE Art Asset Format Specification

Version: 1.0 (derived from flare-engine source, May 2026)

This document specifies how to organize and format **sprites**, **tilesets**, and **maps** so they can be loaded by the [FLARE engine](https://github.com/flareteam/flare-engine) and implemented by other engines supporting the FLARE art format.

Reference implementations:

- **Engine**: [flare-engine](https://github.com/flareteam/flare-engine) — parsers in `AnimationSet.cpp`, `TileSet.cpp`, `Map.cpp`, `MapParallax.cpp`, `FileParser.cpp`
- **Reference assets**: [flare-game](https://github.com/flareteam/flare-game) — e.g. the `fantasycore` and `empyrean_campaign` mods

---

## Table of Contents

1. [Overview](#1-overview)
2. [Common File Format](#2-common-file-format)
3. [Mod Layout and Path Resolution](#3-mod-layout-and-path-resolution)
4. [Image Files](#4-image-files)
5. [Engine Tile Configuration](#5-engine-tile-configuration)
6. [Sprite and Animation Format](#6-sprite-and-animation-format)
7. [Tileset Format](#7-tileset-format)
8. [Map Format](#8-map-format)
9. [Parallax Background Format](#9-parallax-background-format)
10. [Icon Atlas Format](#10-icon-atlas-format)
11. [Coordinate Systems and Rendering Semantics](#11-coordinate-systems-and-rendering-semantics)
12. [Authoring Pipeline](#12-authoring-pipeline)
13. [Validation and Error Handling](#13-validation-and-error-handling)
14. [Appendix A: Value Types](#appendix-a-value-types)
15. [Appendix B: Direction Index Table](#appendix-b-direction-index-table)
16. [Appendix C: Collision Tile Values](#appendix-c-collision-tile-values)
17. [Appendix D: Complete Minimal Examples](#appendix-d-complete-minimal-examples)

---

## 1. Overview

FLARE art assets consist of two parts:

1. **PNG raster images** — atlases containing one or more sprites or tiles
2. **Text definition files** (`.txt`) — INI-style key/value files describing how to slice and place those images

The engine does **not** parse `.tmx` (Tiled) files at runtime. Maps are authored in [Tiled](https://www.mapeditor.org/) and exported to FLARE `.txt` format via the Flare Tiled plugin. Tileset PNGs used during authoring are baked into runtime atlases with companion `tilesetdefs/*.txt` files.

### Asset categories covered by this spec

| Category | Definition path | Image path | Loaded by |
|----------|----------------|------------|-----------|
| Sprites / animations | `animations/**/*.txt` | `images/**/*.png` | `AnimationSet` |
| Tilesets | `tilesetdefs/*.txt` or `tilesets/*.txt` | `images/tilesets/*.png` | `TileSet` |
| Maps | `maps/*.txt` | (via tileset reference) | `Map` |
| Parallax backgrounds | `maps/parallax/*.txt` | `images/parallax/*.png` | `MapParallax` |
| Item icons | `engine/icons.txt` | `images/icons/*.png` | `IconManager` |

> **Note:** Game logic files (`enemies/`, `items/`, `npcs/`, etc.) reference animation paths but are outside the scope of this art-format specification.

---

## 2. Common File Format

All FLARE definition files share a common **INI-style** syntax parsed by `FileParser`.

### 2.1 Syntax

```
# This is a comment. Lines starting with # are ignored.

[section_name]
key=value
another_key=val1,val2,val3
```

Rules:

- **Encoding**: plain text, UTF-8 recommended
- **Line endings**: LF or CRLF
- **Comments**: lines whose first non-whitespace character is `#` are skipped
- **Blank lines**: skipped
- **Sections**: `[section_name]` — sets the active section until the next section header
- **Key/value pairs**: `key=value` — key and value are trimmed; split on the **first** `=`
- **Lists**: comma (`,`) or semicolon (`;`) as separator; the first separator found in a token stream determines the delimiter for that value
- **Keys before any section**: belong to the empty/global section (`section == ""`)

### 2.2 File composition directives

| Directive | Placement | Behavior |
|-----------|-----------|----------|
| `APPEND` | First non-comment, non-blank line of a file | Marks the file as an overlay. When multiple mods provide the same path, files without `APPEND` replace earlier mods entirely; files with `APPEND` are merged on top |
| `INCLUDE otherfile.txt` | Any line (not inside a value) | Inlines the contents of `otherfile.txt` at that point. The included file inherits the current section. Recursive includes of the same file are an error |

### 2.3 Mod override order

When the same relative path exists in multiple mods, mods listed **lower** in `mods/mods.txt` take priority. See [Section 3](#3-mod-layout-and-path-resolution).

---

## 3. Mod Layout and Path Resolution

### 3.1 Mod directory structure

Each mod is a self-contained directory under `mods/`:

```
mods/
└── my_mod/
    ├── settings.txt           # mod metadata (not art)
    ├── engine/
    │   ├── tileset_config.txt # global tile dimensions and orientation
    │   └── icons.txt          # icon atlas registration
    ├── animations/            # sprite animation definitions
    │   ├── hero.txt
    │   ├── enemies/
    │   ├── avatar/
    │   ├── loot/
    │   ├── npcs/
    │   └── powers/
    ├── images/                # PNG atlases
    │   ├── enemies/
    │   ├── avatar/
    │   ├── loot/
    │   ├── tilesets/
    │   ├── icons/
    │   └── parallax/
    ├── tilesetdefs/           # runtime tileset definitions (preferred name in flare-game)
    └── maps/                  # runtime map files
        └── parallax/
```

Alternate path `tilesets/` (instead of `tilesetdefs/`) is equally valid — the engine loads whatever path is referenced by a map's `tileset=` header field.

### 3.2 Path resolution

All paths in definition files (`image=`, `img=`, `tileset=`, etc.) are **relative to the mod root**, using forward slashes:

```
image=images/enemies/zombie.png
tileset=tilesetdefs/tileset_grassland.txt
```

The engine resolves paths by searching active mods from lowest to highest priority until a match is found.

### 3.3 Naming conventions

Observed conventions in reference assets (recommended, not enforced):

- Lowercase **snake_case** filenames: `zombie.txt`, `tileset_grassland.txt`
- Matching basenames between definition and image within a category: `animations/enemies/zombie.txt` ↔ `images/enemies/zombie.png`
- Map names in snake_case: `frontier_plains.txt`, `arrival.txt`

---

## 4. Image Files

### 4.1 Format

- **Preferred format**: PNG with alpha channel
- **Loader**: SDL2_image (`IMG_Load` / `IMG_LoadTexture`)
- Other SDL2_image-supported formats (BMP, GIF, JPG, etc.) may work depending on build configuration, but **PNG is the standard**

### 4.2 Atlas organization

FLARE uses a **definition-driven atlas** model:

- One PNG file typically corresponds to one entity/category (one enemy, one equipment piece, one baked tileset)
- The companion `.txt` file defines sub-rectangles within that PNG
- There is no embedded metadata inside PNG files; all layout information lives in `.txt` definitions

### 4.3 Color and blending

Sprite animations support per-animation `blend_mode` (`normal` or `add`), `alpha_mod` (0–255), and `color_mod` (RGB tint). These apply at render time.

---

## 5. Engine Tile Configuration

**File:** `engine/tileset_config.txt`

This file sets global tile geometry used for map rendering and collision. It is **not** per-tileset — individual tile clips may exceed these dimensions.

```
orientation=isometric
tile_size=64,32
```

| Key | Type | Required | Default | Description |
|-----|------|----------|---------|-------------|
| `orientation` | string | No | `isometric` | `isometric` or `orthogonal` |
| `tile_size` | int, int | No | `64, 32` | Logical tile width and height in pixels |

The engine computes half-dimensions (`tile_w_half`, `tile_h_half`) and uses them for isometric projection (`Utils::mapToScreen`).

> **Non-normative:** Some distributions include a commented `# units_per_tile` line. The current engine does **not** parse this key.

---

## 6. Sprite and Animation Format

**File location:** `animations/**/*.txt`  
**Parser:** `AnimationSet.cpp`  
**Image reference key:** `image`

### 6.1 Purpose

An animation definition describes one or more named animation states (e.g. `stance`, `run`, `die`) for a visual entity. Each state plays a sequence of frames, optionally facing one of **8 fixed directions**.

Animations are referenced by game logic:

```
animations=animations/enemies/zombie.txt
loot_animation=animations/loot/coins5.txt
gfx=leather_chest    # resolves to animations/avatar/{gender}/leather_chest.txt
```

### 6.2 Global keys (empty section)

These keys appear before the first `[section]` or are parsed when `section` is empty:

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `image` | filename[, id] | **Yes** (≥1) | Spritesheet PNG path. Optional second token is an image ID for multi-atlas sets |
| `render_size` | int, int | Required for grid mode | Cell width and height for uncompressed grid layout |
| `render_offset` | int, int | No | Default render offset (foot point) for grid mode |
| `blend_mode` | string | No | `normal` (default) or `add` |
| `alpha_mod` | int | No | Default alpha 0–255 (default 255) |
| `color_mod` | r, g, b | No | RGB tint (default 255, 255, 255) |

Multiple `image=` lines may appear to register several atlases:

```
image=images/creature/body.png, body
image=images/creature/wings.png, wings
```

When a `frame=` line omits the image ID, the first registered image is used.

### 6.3 Animation sections

Each `[animation_name]` block defines one animation state. The section name is the animation identifier used by game logic (e.g. `[run]`, `[melee]`, `[die]`).

The **first section** in the file becomes the default animation when none is specified.

#### Common section keys

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `frames` | int | **Yes** | Total number of animation frames |
| `duration` | duration | **Yes** | Total animation duration (see [Appendix A](#appendix-a-value-types)) |
| `type` | string | **Yes** | `play_once`, `looped`, or `back_forth` |
| `position` | int | Grid mode only | Starting column index in the spritesheet (default 0) |
| `active_frame` | int list or `all` | No | Frames that trigger gameplay events (power hits, hazards) |
| `active_sub_frame` | string | No | When active frames fire within a frame's sub-frame duration: `end` (default), `start`, or `all` |

#### Animation types

| Type | Behavior |
|------|----------|
| `play_once` | Plays forward once and stops on the last frame |
| `looped` | Repeats from the first frame after the last |
| `back_forth` | Plays forward then backward repeatedly; total timeline duration is doubled |

### 6.4 Mode A: Compressed (explicit frames)

**Trigger:** presence of any `frame=` line in a section.

Each `frame=` line defines one frame for one direction:

```
frame=index, direction, x, y, width, height, offset_x, offset_y[, image_id]
```

| Field | Type | Description |
|-------|------|-------------|
| `index` | int | Frame index, 0 to `frames - 1` |
| `direction` | direction | Facing (see [Appendix B](#appendix-b-direction-index-table)) |
| `x`, `y` | int | Top-left corner of the source rectangle in the atlas PNG (pixels) |
| `width`, `height` | int | Source rectangle size (pixels) |
| `offset_x`, `offset_y` | int | Render offset: distance from the **top-left of the drawn sprite** to the entity's **foot point** (map position) |
| `image_id` | string | Optional atlas ID from `image=` registration |

**Requirements:**

- Every frame index SHOULD have entries for all 8 directions for entities that rotate; static effects may use only direction `0`
- Each `(index, direction)` pair MUST be unique
- `index` MUST be in `[0, frames - 1]`

**Example** (loot pickup, single direction):

```
image=images/loot/coins5.png

[power]
frames=6
duration=600ms
type=play_once
frame=0,0,37,56,32,50,7,69
frame=1,0,0,0,40,56,10,84
frame=2,0,0,56,37,59,9,82
frame=3,0,40,0,26,55,8,59
frame=4,0,37,106,17,11,7,7
frame=5,0,54,106,17,9,7,5
```

**Example** (8-direction enemy excerpt):

```
image=images/enemies/zombie.png

[stance]
frames=4
duration=533ms
type=back_forth
frame=0,0,503,191,26,56,11,50
frame=0,1,127,285,31,54,14,51
frame=0,2,397,433,30,53,11,50
...
frame=3,7,405,701,26,55,9,48
```

### 6.5 Mode B: Uncompressed (grid layout)

**Trigger:** section has `frames`, `duration`, and `type` but **no** `frame=` lines. Requires global `render_size`.

Layout algorithm:

- Spritesheet has **8 rows**, one per direction (row 0 = direction 0, row 7 = direction 7)
- Frames for a given direction are placed **horizontally** starting at column `position`
- Frame `i`, direction `d` source rect:
  - `x = render_size.x * (position + i)`
  - `y = render_size.y * d`
  - `w = render_size.x`, `h = render_size.y`
- All frames share `render_offset` from the global key

**Example:**

```
image=images/hero/hero.png
render_size=128,128
render_offset=64,96

[stance]
position=0
frames=4
duration=800ms
type=back_forth
```

> **Limitation:** Uncompressed mode currently uses only the first registered `image=` (multi-atlas uncompressed is not implemented).

### 6.6 Avatar layering

Player appearance composes multiple animation sets from `animations/avatar/{male|female}/`:

- Base body: `default_body.txt`, `default_head.txt`, etc.
- Equipment: one file per piece (`leather_chest.txt`, `shortsword.txt`, …)
- Item definitions reference equipment via short names: `gfx=leather_chest`

Avatar layer files MUST have matching `frames` counts per animation name when validated against a parent set (`AnimationSet::setParent`).

### 6.7 Frame timing

Duration is converted to engine logic frames (see [Appendix A](#appendix-a-value-types)). The engine distributes frames across the duration:

- If `duration % frames == 0`, each frame gets equal sub-frame time
- Otherwise, frames are distributed using a Bresenham-like algorithm

For `back_forth`, the effective timeline length is doubled.

---

## 7. Tileset Format

**File location:** `tilesetdefs/*.txt` or `tilesets/*.txt`  
**Parser:** `TileSet.cpp`  
**Image reference key:** `img`

### 7.1 Purpose

A tileset definition maps **numeric tile IDs** to sub-rectangles within one or more atlas PNGs. Map layer data stores these IDs. ID **0 always means empty** (no tile rendered).

Tile IDs correspond to **Tiled global tile IDs (GIDs)** in the authoring pipeline. The reference game uses GIDs starting at 16 for biome floor tiles.

### 7.2 File structure

A tileset file contains one or more `[tileset]` sections (or keys in the empty section for a single implicit tileset). Each section references one image atlas.

```
img=images/tilesets/tileset_grassland.png

tile=16,288,649,64,32,32,16
tile=17,788,336,64,32,32,16
...
animation=265,406,290,66ms,367,354,66ms,367,418,66ms,367,482,66ms
```

When multiple `[tileset]` sections exist, each declares its own `img=` and contributes tiles to the same ID namespace.

### 7.3 Keys

| Key | Scope | Required | Description |
|-----|-------|----------|-------------|
| `img` | per `[tileset]` section | **Yes** | Path to tile atlas PNG |
| `tile` | repeatable | **Yes** (per defined tile) | Single tile definition |
| `animation` | repeatable | No | Animated tile cycle |

#### `tile=` format

```
tile=index, x, y, width, height, offset_x, offset_y
```

| Field | Type | Description |
|-------|------|-------------|
| `index` | int | Tile ID used in map layers (≥ 1; 0 reserved) |
| `x`, `y` | int | Top-left of tile graphic in atlas (pixels) |
| `width`, `height` | int | Clip size (pixels); may differ from `tile_size` |
| `offset_x`, `offset_y` | int | Anchor offset from clip top-left to the tile's ground point |

The tile ID space is **sparse**: the vector grows to accommodate the highest index. Undefined indices behave as empty.

**Typical dimensions** (isometric, 64×32 logical tile):

| Tile kind | Typical clip | Typical offset |
|-----------|-------------|----------------|
| Flat floor | 64×32 | 32, 16 (center of diamond) |
| Small prop | 64×64 – 64×128 | 32, height − 16 |
| Large tree | 128×256 | −32, 0 to 32, 80 |

#### `animation=` format

```
animation=index, x1, y1, duration1, x2, y2, duration2, ...
```

| Field | Description |
|-------|-------------|
| `index` | Tile ID to animate |
| `xN, yN` | Atlas coordinates for frame N (replaces clip origin; size stays from `tile=` definition) |
| `durationN` | Frame display duration (`ms` or `s`) |

Animations cycle continuously. The initial clip from the `tile=` entry is overwritten at runtime.

**Example:**

```
animation=265,406,290,66ms,367,354,66ms,367,418,66ms,367,482,66ms
```

### 7.4 Runtime baking

In the reference pipeline, many Tiled source PNGs are baked into a single `images/tilesets/tileset_<biome>.png` atlas. The `tilesetdefs/tileset_<biome>.txt` file maps every GID to its position in that baked atlas. This indirection is transparent to the map format — maps only store numeric IDs.

---

## 8. Map Format

**File location:** `maps/*.txt`  
**Parser:** `Map.cpp`

### 8.1 Purpose

Maps define tile layers, collision, entity spawn points, and scripted events. Visual tile layers reference a tileset definition via the header.

### 8.2 Sections overview

| Section | Repeatable | Purpose |
|---------|------------|---------|
| `[header]` | No | Map metadata and tileset reference |
| `[tilesets]` | No | Tiled re-import metadata (**not read at runtime**) |
| `[layer]` | Yes | Tile layer data |
| `[enemy]` | Yes | Enemy spawn groups |
| `[npc]` | Yes | NPC placements |
| `[event]` | Yes | Map events / triggers |

This specification covers `[header]`, `[tilesets]`, and `[layer]` — the art-relevant sections.

### 8.3 `[header]` keys

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `width` | int | **Yes** | Map width in tiles (minimum 1) |
| `height` | int | **Yes** | Map height in tiles (minimum 1) |
| `tileset` | filename | **Yes** | Path to tileset definition (e.g. `tilesetdefs/tileset_grassland.txt`) |
| `title` | string | No | Display name (localized via message engine) |
| `hero_pos` | int, int | Strongly recommended | Player spawn tile coordinates; engine adds 0.5 to center within tile |
| `music` | filename | No | Background music path |
| `parallax_layers` | filename | No | Parallax definition (e.g. `maps/parallax/fog.txt`) |
| `background_color` | r, g, b, a | No | Clear/fill color |
| `fogofwar` | int | No | 0=off, 1=minimap, 2=tint, 3=overlay |
| `save_fogofwar` | bool | No | Persist fog-of-war state for this map |
| `tilewidth` | int | No | **Ignored at runtime** — preserved for Tiled re-import |
| `tileheight` | int | No | **Ignored at runtime** — preserved for Tiled re-import |
| `orientation` | string | No | **Ignored at runtime** — preserved for Tiled re-import |

**Example:**

```
[header]
width=40
height=40
tilewidth=64
tileheight=32
orientation=isometric
background_color=0,0,0,255
hero_pos=20,20
music=music/wind_ambient.ogg
tileset=tilesetdefs/tileset_grassland.txt
title=Arrival
```

### 8.4 `[tilesets]` section (export metadata only)

Preserved by the Tiled export plugin for round-trip editing. **The engine does not parse this section at runtime.**

```
[tilesets]
tileset=path/to/source.png,tile_w,tile_h,offset_x,offset_y
```

Each line maps a Tiled source tileset image to its dimensions and draw offset.

### 8.5 `[layer]` sections

Each layer is a separate `[layer]` section:

```
[layer]
type=background
format=dec
data=
16,17,18,19,0,0,
20,21,22,23,0,0,
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `type` | string | **Yes** | Layer name (determines rendering role) |
| `format` | string | No | If present, MUST be `dec`. Omitting is valid |
| `data` | raw CSV | **Yes** | Comma-separated tile IDs |

#### Layer data format

After `data=`, the next **`height` lines** contain the tile data:

- Each line is comma-separated integers
- Each line contains exactly **`width`** values
- Trailing comma is optional (engine appends one if missing)
- Values are read row by row: line 1 = y=0, line 2 = y=1, …
- Within a row: first value = x=0, second = x=1, …
- `0` = no tile

#### Special layer names

| Layer name | Role |
|------------|------|
| `collision` | Collision grid (not rendered). If absent, an all-zero layer is created |
| `object` | Split point: layers **below** render as background; this layer and above render as foreground |
| `fow_dark`, `fow_fog` | Fog-of-war overlay layers |
| Any other name | Visual tile layer, rendered in file order |

**Layer ordering:** Layers render in the order they appear in the file. The `object` layer index determines the background/foreground entity split.

**Collision layer values:** See [Appendix C](#appendix-c-collision-tile-values). These are **not** tileset GIDs — they are small integers 0–8.

**Example layer excerpt:**

```
[layer]
type=background
data=
0,0,0,17,18,19,16,
0,0,0,20,21,22,23,

[layer]
type=object
data=
0,0,0,60,49,53,53,

[layer]
type=collision
data=
0,0,0,0,0,0,0,
0,0,0,0,1,1,0,
```

---

## 9. Parallax Background Format

**File location:** `maps/parallax/*.txt`  
**Parser:** `MapParallax.cpp`  
**Referenced from map header:** `parallax_layers=maps/parallax/fog.txt`

Each `[layer]` section defines one scrolling background:

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `image` | filename | **Yes** | Full-screen tileable PNG |
| `speed` | float | No | Scroll speed relative to camera (default 0) |
| `fixed_speed` | float, float | No | Camera-independent scroll speed (x, y) |
| `map_layer` | string | No | Render this parallax layer after the named map tile layer |

**Example:**

```
[layer]
image=images/parallax/fog.png
speed=0.01
fixed_speed=-0.0025,-0.005
map_layer=object
```

Parallax images are tiled to fill the viewport. Rendering can be disabled globally via engine settings (`parallax_layers`).

---

## 10. Icon Atlas Format

**File:** `engine/icons.txt`  
**Parser:** `IconManager.cpp`

Icons are uniform grids used for inventory UI. Not map/sprite art, but part of the FLARE visual asset system.

```
icon_set=0,images/icons/icons.png
icon_set=1024,images/icons/icons_overlay.png
text_offset=0,0
```

| Key | Type | Description |
|-----|------|-------------|
| `icon_set` | int, filename | First icon index and atlas PNG path |
| `text_offset` | int, int | Pixel offset for quantity text overlay |

Grid cell size comes from `engine/resolutions.txt` → `icon_size`. Icons are indexed sequentially left-to-right, top-to-bottom starting at `first_id`.

If `engine/icons.txt` is absent, the engine falls back to `images/icons/icons.png` starting at ID 0.

---

## 11. Coordinate Systems and Rendering Semantics

### 11.1 Map coordinates

- Map positions are in **tile units** (integer grid)
- Entity positions use tile coordinates + 0.5 (center of tile) for spawn points (`hero_pos`, NPC `location`, etc.)

### 11.2 Screen projection

**Isometric** (default):

- Logical tile size from `engine/tileset_config.txt` (`tile_size`)
- `Utils::mapToScreen(map_x, map_y)` converts tile coordinates to screen pixels
- Tile draw position: `screen_pos - tile.offset`
- Layer rendering iterates in an isometric order (SW corner of each tile cell)

**Orthogonal:**

- Same offset subtraction; grid iteration differs

### 11.3 Sprite render offset

For animations, `(offset_x, offset_y)` on each frame defines the **foot point**:

```
screen_draw_x = entity_screen_x - offset_x
screen_draw_y = entity_screen_y - offset_y
source_rect = (x, y, width, height) from frame definition
```

The foot point aligns with the entity's map position. For tall sprites, `offset_y` is typically near the bottom of the clip (feet).

### 11.4 Tile render offset

Similarly for tiles:

```
dest.x = tile_screen_x - offset_x
dest.y = tile_screen_y - offset_y
```

For isometric floor tiles with clip 64×32, offset `(32, 16)` places the diamond center at the tile anchor.

### 11.5 Directions

All directional sprites use exactly **8 directions**, hard-coded engine-wide (`Animation::DIRECTIONS = 8`). See [Appendix B](#appendix-b-direction-index-table).

---

## 12. Authoring Pipeline

Recommended workflow used by the FLARE project:

```
┌─────────────────────────┐     ┌──────────────────────────┐
│ art_src/ (Blender, etc.)│────►│ images/ + animations/    │
│ Source character art    │ bake│ Runtime sprite atlases   │
└─────────────────────────┘     └──────────────────────────┘

┌─────────────────────────┐     ┌──────────────────────────┐
│ tiled/<biome>/*.tmx     │────►│ maps/*.txt               │
│ Tiled map editor        │export│ Runtime map files        │
└─────────────────────────┘     └──────────────────────────┘
         │                                    │
         ▼                                    ▼
┌─────────────────────────┐     ┌──────────────────────────┐
│ tiled/<biome>/*.png     │────►│ tilesetdefs/*.txt        │
│ Source tile images      │ bake│ + images/tilesets/*.png  │
└─────────────────────────┘     └──────────────────────────┘
```

1. **Sprites:** Render or draw atlases → write `animations/*.txt` with `frame=` lines and offsets
2. **Tilesets:** Author in Tiled with per-biome tileset PNGs → bake to single atlas → generate `tilesetdefs/*.txt` mapping GIDs to atlas rects
3. **Maps:** Author in Tiled with Flare plugin → export to `maps/*.txt`
4. **Distribution:** Ship `mods/` directory; `art_src/` and `tiled/` are development-only

### Tiled plugin

The Flare Tiled plugin exports:

- Layer data as decimal CSV (`format=dec` if written)
- `[tilesets]` metadata for re-import
- Header fields (`tilewidth`, `tileheight`, `orientation`)
- Object layers as `[enemy]`, `[npc]`, `[event]` sections

---

## 13. Validation and Error Handling

Implementors SHOULD enforce:

| Check | Consequence if violated |
|-------|------------------------|
| Map layer row width ≠ `header.width` | Parse error, map load failure |
| Map layer row count ≠ `header.height` | Incomplete/wrong data |
| `layer.format` present and ≠ `dec` | Parse error |
| Animation `frame` index ≥ `frames` | Parse error |
| Map tile ID not in tileset | Tile silently replaced with 0, error logged |
| Missing `hero_pos` | Warning; defaults to (0, 0) |
| Missing `collision` layer | All-walkable collision created |
| Avatar layer frame count ≠ parent | Parse error (frame count corrected to parent) |

---

## Appendix A: Value Types

| Type | Format | Examples |
|------|--------|----------|
| `int` | Signed integer | `42`, `-1` |
| `float` | Decimal | `0.01`, `-0.5` |
| `bool` | Boolean | `true`, `false`, `1`, `0` |
| `point` | int, int | `20, 30` |
| `rectangle` | int, int, int, int | `10, 20, 64, 32` |
| `color` | int, int, int | `255, 128, 0` |
| `color+alpha` | int, int, int, int | `0, 0, 0, 255` |
| `duration` | int + suffix | `800ms`, `1s`, `533ms` |
| `direction` | name or int 0–7 | `NE`, `3`, `0` |
| `filename` | mod-relative path | `images/enemies/zombie.png` |

### Duration conversion

Durations convert to logic frames:

- `{N}s` → `N * max_frames_per_sec`
- `{N}ms` → round(`N * max_frames_per_sec / 1000`), minimum 1
- No suffix → treated as milliseconds with warning

Default `max_frames_per_sec` is engine-configurable (typically 60).

---

## Appendix B: Direction Index Table

8 directions, counter-clockwise from SW:

| Index | Name | Int alias |
|-------|------|-----------|
| 0 | SW | ↙ |
| 1 | W | ← |
| 2 | NW | ↖ |
| 3 | N | ↑ |
| 4 | NE | ↗ |
| 5 | E | → |
| 6 | SE | ↘ |
| 7 | S | ↓ |

Both integer indices and compass names (`N`, `NE`, `E`, `SE`, `S`, `SW`, `W`, `NW`) are accepted in `frame=` lines and other direction fields.

---

## Appendix C: Collision Tile Values

Used in the `collision` map layer (not tileset GIDs):

| Value | Name | Meaning |
|-------|------|---------|
| 0 | BLOCKS_NONE | Walkable |
| 1 | BLOCKS_ALL | Blocks all movement |
| 2 | BLOCKS_MOVEMENT | Blocks ground units; flying units pass |
| 3 | BLOCKS_ALL_HIDDEN | Like 1, hidden on minimap |
| 4 | BLOCKS_MOVEMENT_HIDDEN | Like 2, hidden on minimap |
| 5 | MAP_ONLY | Map-only collision |
| 6 | MAP_ONLY_ALT | Map-only collision (alternate) |
| 7 | BLOCKS_ENTITIES | Entity blocking |
| 8 | BLOCKS_ENEMIES | Ally standing; hero may pass depending on settings |

These values match Tiled's Flare collision tileset export.

---

## Appendix D: Complete Minimal Examples

### D.1 Minimal tileset

```
img=images/tilesets/my_tileset.png

tile=1,0,0,64,32,32,16
tile=2,64,0,64,32,32,16
```

### D.2 Minimal map

```
[header]
width=3
height=2
tileset=tilesetdefs/my_tileset.txt
hero_pos=1,1

[layer]
type=background
data=
1,2,1,
2,1,2,

[layer]
type=collision
data=
0,0,0,
0,0,0,
```

### D.3 Minimal sprite (single-direction effect)

```
image=images/effects/spark.png

[play]
frames=4
duration=400ms
type=play_once
frame=0,0,0,0,16,16,8,8
frame=1,0,16,0,16,16,8,8
frame=2,0,32,0,16,16,8,8
frame=3,0,48,0,16,16,8,8
```

### D.4 Minimal parallax

```
[layer]
image=images/parallax/sky.png
speed=0.05
map_layer=background
```

---

## Rust Reference Implementation

An engine-agnostic Rust crate [`flare-format`](../rust/flare-format/) is provided for loading FLARE art assets without rendering dependencies. It parses animations, tilesets, maps, parallax layers, and icons into typed structs, and includes runtime helpers for animation playback, tile animation cycling, coordinate conversion, and collision walkability.

---

## Document History

| Version | Date | Notes |
|---------|------|-------|
| 1.0 | 2026-05-29 | Initial specification derived from flare-engine parsers and flare-game reference assets |

---

## License

This specification documents the FLARE engine asset format. Art and data files distributed with this repository are licensed under **CC-BY-SA 3.0**. See [LICENSE.txt](../LICENSE.txt) and [CREDITS.txt](../CREDITS.txt).
