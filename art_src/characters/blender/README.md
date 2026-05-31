# FLARE 8-direction Blender render pipeline

Automates what the legacy [`render8dirs.py`](../render8dirs.py) script does in Blender 2.79, updated for **Blender 4.x / 5.x** headless use with **Mixamo FBX** imports.

**No external Python packages.** Rendering, FBX import, and spritesheet assembly all run inside Blender's bundled Python (numpy included in official builds).

## Blender setup

Put a Blender binary on your `PATH`, or symlink one:

```bash
ln -s /path/to/blender-5.1.2-linux-x64/blender ~/.local/bin/blender
```

For a one-off run without changing `PATH`:

```bash
BLENDER=/path/to/blender ./run_render8dirs.sh ...
```

The Debian-packaged Blender is often incomplete for FBX; use an official build from blender.org if imports fail.

## What it produces

For each animation frame and each of eight FLARE facings (SW → S, CCW), PNG files:

```text
walk0_0001.png  … walk0_0035.png   # direction 0 (SW)
walk1_0001.png  …                  # direction 1 (W)
…
walk7_0001.png  …                  # direction 7 (S)
```

Optionally a single **uncompressed grid** spritesheet (8 rows × N frame columns) plus a starter `animations/.../*.txt` definition.

## Quick start (Mixamo walk, 35 frames)

```bash
cd art_src/characters/blender

./run_render8dirs.sh ~/Downloads/YBot_Walk.fbx /tmp/walk_out \
  --prefix walk \
  --frames 35 \
  --cell-size 128 \
  --build-sheet /tmp/walk_out/walk.png \
  --build-txt /tmp/walk_out/walk.txt
```

Or invoke Blender directly:

```bash
blender --background --python flare_render_pipeline.py -- \
  --fbx ~/Downloads/YBot_Walk.fbx \
  --output-dir /tmp/walk_out \
  --prefix walk \
  --frames 35 \
  --build-sheet /tmp/walk_out/walk.png \
  --build-txt /tmp/walk_out/walk.txt
```

## Interactive workflow (GUI Blender)

1. **File → New → General**
2. **Scripting** workspace → open `setup_flare_render_scene.py` → Run Script  
   Creates `RenderPlatform`, orthographic isometric camera, and lights.
3. **File → Import → FBX** (your Mixamo file), or run `import_mixamo.py` from the text editor with paths edited.
4. Set timeline **Start=1**, **End=35** (Mixamo often leaves a longer default range).
5. Open `render8dirs.py`, set `output_dir` / `name_prefix` at the bottom, Run Script.

## Camera / direction model

Same as upstream FLARE:

- Fixed **orthographic** camera pitched ~60° with 45° yaw (2:1 game iso).
- Camera and lights stay in **world space** (not parented to ``RenderPlatform``).
- Each direction rotates only ``RenderPlatform`` (character + look target) **−45° on Z before rendering**.
- Direction index `0..7` matches FLARE rows: SW, W, NW, N, NE, E, SE, S.

Tune framing with `--no-auto-fit-camera --ortho-scale 2.0` if auto-fit leaves the character too large/small.

**Blank white PNGs** usually mean the camera missed the model entirely (fully transparent RGBA). The pipeline now auto-aims the camera at the imported mesh.

## CLI reference

| Flag | Default | Purpose |
|------|---------|---------|
| `--fbx` | (required*) | Mixamo FBX path (*unless `--skip-render`) |
| `--output-dir` | (required) | PNG output folder |
| `--prefix` | `walk` | Filename prefix |
| `--frames` | `35` | Frame count (fixes Mixamo range) |
| `--frame-start` | `1` | First frame |
| `--animation` | auto | Action name substring |
| `--cell-size` | `128` | Render resolution (square) |
| `--ortho-scale` | `2.4` | Manual ortho scale (with `--no-auto-fit-camera`) |
| `--no-auto-fit-camera` | — | Disable auto-fit; use `--ortho-scale` |
| `--packing` | `compressed` | `compressed` (tight atlas + `frame=` lines) or `grid` |
| `--build-sheet` | — | Spritesheet PNG path |
| `--build-txt` | — | FLARE animation definition |
| `--image-mod-path` | — | Mod-relative `image=` path (required with `--build-txt`) |
| `--render-offset` | `64,96` | Foot point in source render cell |
| `--character-z-deg` | `0` | Optional post-import Z rotation to tune Mixamo facing |
| `--run-fps` | `15` | Run playback in animation frames/sec (`duration = frames/run_fps`) |
| `--skip-render` | — | Rebuild sheet/txt from existing PNGs |

Rebuild spritesheet only (PNG sequences already on disk):

```bash
./publish_character_to_mod.sh prince walk

# or manually:
./run_render8dirs.sh --build-only art_src/characters/blender/render \
  --prefix walk \
  --build-sheet mods/fantasycore/images/avatar/prince/walk.png \
  --build-txt mods/fantasycore/animations/avatar/prince/walk.txt \
  --image-mod-path images/avatar/prince/walk.png
```

The generated `.txt` uses **compressed packing** by default: alpha-trimmed sprites, shelf-packed into one atlas, with explicit `frame=` lines and per-frame `offset_x/offset_y`. It includes `[stance]` and `[run]` for the game client.

Use `--packing grid` for the legacy uniform 128×128 grid layout (uncompressed mode with `render_size`).

## Tuning tips

- **Feet not at cell anchor**: adjust armature location in Blender, then update `render_offset` in the `.txt` (typically `(cell_width/2, cell_height*0.75)`).
- **Wrong clip**: pass `--animation Walk` (substring match).
- **Legacy `.blend` from flare-game**: open in Blender 2.79, or rebuild the rig with `setup_flare_render_scene.py` and re-parent your mesh.
- **Ground plane visible**: enable **Film → Transparent** (already set by setup script); hide any ground mesh before rendering.

## Files

| File | Role |
|------|------|
| `flare_constants.py` | Shared camera/direction constants |
| `setup_flare_render_scene.py` | Create rig + render settings |
| `import_mixamo.py` | FBX import, frame range, parenting |
| `render8dirs.py` | Eight-direction animation render |
| `flare_render_pipeline.py` | Headless orchestrator |
| `flare_spritesheet.py` | Grid montage + `.txt` stub (`bpy`) |
| `build_spritesheet.py` | Spritesheet-only Blender entry point |
| `publish_character_to_mod.sh` | Publish render frames → fantasycore mod paths |
| `run_render8dirs.sh` | Shell wrapper (`PATH` / `BLENDER`) |
