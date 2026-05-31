"""FLARE spritesheet assembly using Blender's bundled Python (``bpy``)."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

import bpy

from flare_constants import DEFAULT_CELL_SIZE, DEFAULT_RENDER_OFFSET

FRAME_PATTERN = re.compile(
    r"^(?P<prefix>.+?)(?P<dir>[0-7])_(?P<frame>\d+)\.png$",
    re.IGNORECASE,
)

DIRECTION_COUNT = 8
PACK_PADDING = 1
ALPHA_THRESHOLD = 1.0 / 255.0


@dataclass(frozen=True)
class AnimationSection:
    name: str
    frames: int
    duration_ms: int
    animation_type: str = "looped"


@dataclass(frozen=True)
class TrimmedSprite:
    direction: int
    frame_index: int
    trim_left: int
    trim_top: int
    trim_right: int
    trim_bottom: int
    offset_x: int
    offset_y: int
    pixels: tuple[float, ...]
    source_w: int
    source_h: int

    @property
    def width(self) -> int:
        return self.trim_right - self.trim_left

    @property
    def height(self) -> int:
        return self.trim_bottom - self.trim_top

    @property
    def key(self) -> tuple[int, int]:
        return self.frame_index, self.direction


@dataclass(frozen=True)
class PackedSprite:
    direction: int
    frame_index: int
    atlas_x: int
    atlas_y: int
    width: int
    height: int
    offset_x: int
    offset_y: int


@dataclass(frozen=True)
class CompressedAtlas:
    width: int
    height: int
    pixels: list[float]
    placements: dict[tuple[int, int], PackedSprite]


def discover_frames(input_dir: Path, name_prefix: str) -> dict[int, dict[int, Path]]:
    """Return ``{direction: {frame_number: path}}``."""
    grouped: dict[int, dict[int, Path]] = {i: {} for i in range(DIRECTION_COUNT)}

    for path in sorted(input_dir.glob("*.png")):
        match = FRAME_PATTERN.match(path.name)
        if not match or match.group("prefix") != name_prefix:
            continue
        direction = int(match.group("dir"))
        frame = int(match.group("frame"))
        grouped.setdefault(direction, {})[frame] = path

    return grouped


def _frame_index_map(grouped: dict[int, dict[int, Path]]) -> tuple[list[int], dict[int, int]]:
    frame_numbers = sorted({frame for direction in range(DIRECTION_COUNT) for frame in grouped.get(direction, {})})
    if not frame_numbers:
        raise RuntimeError("No frame numbers discovered")
    index_by_number = {frame_number: index for index, frame_number in enumerate(frame_numbers)}
    return frame_numbers, index_by_number


def run_animation_duration_ms(frame_count: int, run_fps: float) -> int:
    """Total loop duration for ``frames`` sprite indices at ``run_fps`` frames per second."""
    if frame_count <= 0:
        raise ValueError("frame_count must be positive")
    if run_fps <= 0:
        raise ValueError("run_fps must be positive")
    return int(round(1000.0 * frame_count / run_fps))


def default_game_sections(run_frame_count: int, *, run_fps: float = 15.0) -> list[AnimationSection]:
    """Sections expected by rust-spacetimedb-game ``CharacterLibrary``."""
    run_duration_ms = run_animation_duration_ms(run_frame_count, run_fps)
    return [
        AnimationSection("stance", frames=1, duration_ms=800, animation_type="looped"),
        AnimationSection("run", frames=run_frame_count, duration_ms=run_duration_ms, animation_type="looped"),
    ]


def _trim_bounds(
    pixels: tuple[float, ...] | list[float],
    width: int,
    height: int,
) -> tuple[int, int, int, int]:
    """Return ``left, top, right, bottom`` in top-left image coordinates."""
    min_x = width
    min_y = height
    max_x = -1
    max_y = -1

    for row in range(height):
        blender_row = height - 1 - row
        for col in range(width):
            alpha = pixels[(blender_row * width + col) * 4 + 3]
            if alpha <= ALPHA_THRESHOLD:
                continue
            min_x = min(min_x, col)
            max_x = max(max_x, col)
            min_y = min(min_y, row)
            max_y = max(max_y, row)

    if max_x < 0:
        return 0, 0, width, height

    return min_x, min_y, max_x + 1, max_y + 1


def _load_trimmed_sprite(
    path: Path,
    *,
    direction: int,
    frame_index: int,
    foot_point: tuple[int, int],
) -> TrimmedSprite:
    image = bpy.data.images.load(str(path.resolve()))
    try:
        width, height = image.size[0], image.size[1]
        pixels = tuple(image.pixels)
        left, top, right, bottom = _trim_bounds(pixels, width, height)
        return TrimmedSprite(
            direction=direction,
            frame_index=frame_index,
            trim_left=left,
            trim_top=top,
            trim_right=right,
            trim_bottom=bottom,
            offset_x=foot_point[0] - left,
            offset_y=foot_point[1] - top,
            pixels=pixels,
            source_w=width,
            source_h=height,
        )
    finally:
        bpy.data.images.remove(image)


def _shelf_pack(sprites: list[TrimmedSprite], *, padding: int = PACK_PADDING, max_width: int = 2048) -> dict[tuple[int, int], tuple[int, int]]:
    """Return map of sprite key -> atlas top-left position."""
    ordered = sorted(sprites, key=lambda sprite: (sprite.width * sprite.height, sprite.height), reverse=True)
    placements: dict[tuple[int, int], tuple[int, int]] = {}
    cursor_x = 0
    cursor_y = 0
    row_height = 0

    for sprite in ordered:
        width = sprite.width
        height = sprite.height
        if cursor_x > 0 and cursor_x + width > max_width:
            cursor_x = 0
            cursor_y += row_height + padding
            row_height = 0

        placements[sprite.key] = (cursor_x, cursor_y)
        cursor_x += width + padding
        row_height = max(row_height, height)

    return placements


def _blit_trimmed_top_left(
    dst_pixels: list[float],
    dst_w: int,
    dst_h: int,
    sprite: TrimmedSprite,
    dst_x: int,
    dst_y: int,
) -> None:
    for row in range(sprite.height):
        src_row_bl = sprite.source_h - 1 - (sprite.trim_top + row)
        dst_row_bl = dst_h - 1 - (dst_y + row)
        for col in range(sprite.width):
            src_col = sprite.trim_left + col
            si = (src_row_bl * sprite.source_w + src_col) * 4
            di = (dst_row_bl * dst_w + (dst_x + col)) * 4
            dst_pixels[di : di + 4] = sprite.pixels[si : si + 4]


def build_compressed_spritesheet(
    input_dir: Path,
    output_png: Path,
    *,
    name_prefix: str,
    foot_point: tuple[int, int] = DEFAULT_RENDER_OFFSET,
    padding: int = PACK_PADDING,
) -> CompressedAtlas:
    grouped = discover_frames(input_dir, name_prefix)
    frame_numbers, index_by_number = _frame_index_map(grouped)

    sprites: list[TrimmedSprite] = []
    for direction in range(DIRECTION_COUNT):
        for frame_number in frame_numbers:
            src_path = grouped.get(direction, {}).get(frame_number)
            if src_path is None:
                continue
            sprites.append(
                _load_trimmed_sprite(
                    src_path,
                    direction=direction,
                    frame_index=index_by_number[frame_number],
                    foot_point=foot_point,
                )
            )

    if not sprites:
        raise RuntimeError("No sprites loaded for compressed packing")

    positions = _shelf_pack(sprites, padding=padding)
    atlas_w = max(pos[0] + sprite.width for sprite in sprites if (pos := positions[sprite.key]))
    atlas_h = max(pos[1] + sprite.height for sprite in sprites if (pos := positions[sprite.key]))

    dst_pixels = [0.0] * (atlas_w * atlas_h * 4)
    placements: dict[tuple[int, int], PackedSprite] = {}

    for sprite in sprites:
        atlas_x, atlas_y = positions[sprite.key]
        _blit_trimmed_top_left(dst_pixels, atlas_w, atlas_h, sprite, atlas_x, atlas_y)
        packed = PackedSprite(
            direction=sprite.direction,
            frame_index=sprite.frame_index,
            atlas_x=atlas_x,
            atlas_y=atlas_y,
            width=sprite.width,
            height=sprite.height,
            offset_x=sprite.offset_x,
            offset_y=sprite.offset_y,
        )
        placements[sprite.key] = packed

    sheet = bpy.data.images.new(
        "FlareCompressedAtlas",
        width=atlas_w,
        height=atlas_h,
        alpha=True,
        float_buffer=False,
    )
    sheet.pixels = dst_pixels
    output_png.parent.mkdir(parents=True, exist_ok=True)
    sheet.filepath_raw = str(output_png.resolve())
    sheet.file_format = "PNG"
    sheet.save()
    bpy.data.images.remove(sheet)

    return CompressedAtlas(width=atlas_w, height=atlas_h, pixels=dst_pixels, placements=placements)


def write_compressed_animation_def(
    output_txt: Path,
    *,
    image_mod_path: str,
    sections: list[AnimationSection],
    placements: dict[tuple[int, int], PackedSprite],
) -> None:
    lines = [f"image={image_mod_path}", ""]

    for index, section in enumerate(sections):
        if index > 0:
            lines.append("")
        lines.extend(
            [
                f"[{section.name}]",
                f"frames={section.frames}",
                f"duration={section.duration_ms}ms",
                f"type={section.animation_type}",
            ]
        )

        for frame_index in range(section.frames):
            for direction in range(DIRECTION_COUNT):
                packed = placements.get((frame_index, direction))
                if packed is None:
                    raise RuntimeError(
                        f"Missing packed frame for section={section.name} "
                        f"frame={frame_index} direction={direction}"
                    )
                lines.append(
                    "frame="
                    f"{frame_index},{direction},"
                    f"{packed.atlas_x},{packed.atlas_y},"
                    f"{packed.width},{packed.height},"
                    f"{packed.offset_x},{packed.offset_y}"
                )

    output_txt.parent.mkdir(parents=True, exist_ok=True)
    output_txt.write_text("\n".join(lines) + "\n", encoding="utf-8")


def _blit_cell(
    dst_pixels: list[float],
    dst_w: int,
    dst_h: int,
    src_pixels: list[float],
    src_w: int,
    src_h: int,
    dst_x: int,
    dst_y_top: int,
    cell_size: int,
) -> None:
    copy_w = min(cell_size, src_w)
    copy_h = min(cell_size, src_h)

    for row in range(copy_h):
        src_row = copy_h - 1 - row
        for col in range(copy_w):
            si = (src_row * src_w + col) * 4
            dst_row = dst_h - 1 - (dst_y_top + row)
            di = (dst_row * dst_w + dst_x + col) * 4
            dst_pixels[di : di + 4] = src_pixels[si : si + 4]


def build_grid_spritesheet(
    input_dir: Path,
    output_png: Path,
    *,
    name_prefix: str,
    cell_size: int = DEFAULT_CELL_SIZE,
) -> int:
    grouped = discover_frames(input_dir, name_prefix)
    active_dirs = [d for d in range(DIRECTION_COUNT) if grouped.get(d)]
    if not active_dirs:
        raise RuntimeError(
            f"No frames found in {input_dir} with prefix '{name_prefix}' and pattern "
            f"'{name_prefix}<dir>_<frame>.png'"
        )

    frame_numbers = sorted({frame for direction in active_dirs for frame in grouped[direction]})
    if not frame_numbers:
        raise RuntimeError("No frame numbers discovered")

    cols = len(frame_numbers)
    sheet_w = cols * cell_size
    sheet_h = DIRECTION_COUNT * cell_size

    sheet = bpy.data.images.new(
        "FlareSpritesheet",
        width=sheet_w,
        height=sheet_h,
        alpha=True,
        float_buffer=False,
    )
    dst_pixels = [0.0] * (sheet_w * sheet_h * 4)

    loaded: list[bpy.types.Image] = []
    try:
        for direction in range(DIRECTION_COUNT):
            for col_index, frame in enumerate(frame_numbers):
                src_path = grouped.get(direction, {}).get(frame)
                if src_path is None:
                    continue

                src = bpy.data.images.load(str(src_path.resolve()))
                loaded.append(src)
                src_pixels = list(src.pixels)
                _blit_cell(
                    dst_pixels,
                    sheet_w,
                    sheet_h,
                    src_pixels,
                    src.size[0],
                    src.size[1],
                    col_index * cell_size,
                    direction * cell_size,
                    cell_size,
                )
    finally:
        for image in loaded:
            bpy.data.images.remove(image)

    sheet.pixels = dst_pixels
    output_png.parent.mkdir(parents=True, exist_ok=True)
    sheet.filepath_raw = str(output_png.resolve())
    sheet.file_format = "PNG"
    sheet.save()
    bpy.data.images.remove(sheet)

    return cols


def assemble_flare_output(
    input_dir: Path,
    *,
    name_prefix: str,
    cell_size: int,
    output_png: Path | None,
    output_txt: Path | None,
    image_mod_path: str | None,
    render_offset: tuple[int, int],
    run_fps: float,
    packing: str = "compressed",
    sections: list[AnimationSection] | None = None,
) -> int:
    grouped = discover_frames(input_dir, name_prefix)
    frame_numbers, _index_by_number = _frame_index_map(grouped)
    frame_count = len(frame_numbers)

    animation_sections = sections or default_game_sections(frame_count, run_fps=run_fps)
    placements: dict[tuple[int, int], PackedSprite] = {}

    if output_png is not None:
        if packing == "compressed":
            atlas = build_compressed_spritesheet(
                input_dir,
                output_png,
                name_prefix=name_prefix,
                foot_point=render_offset,
            )
            placements = atlas.placements
            print(
                f"Wrote {output_png} ({atlas.width}x{atlas.height} compressed atlas, "
                f"{len(placements)} sprites)",
                flush=True,
            )
        elif packing == "grid":
            build_grid_spritesheet(
                input_dir,
                output_png,
                name_prefix=name_prefix,
                cell_size=cell_size,
            )
            print(f"Wrote {output_png} ({frame_count} frames x {DIRECTION_COUNT} directions)", flush=True)
        else:
            raise RuntimeError(f"Unknown packing mode: {packing}")

    if output_txt is not None:
        if image_mod_path is None:
            raise RuntimeError("--image-mod-path is required when writing a FLARE animation definition")

        if packing == "compressed":
            if not placements:
                atlas = build_compressed_spritesheet(
                    input_dir,
                    output_png or (input_dir / f"{name_prefix}.png"),
                    name_prefix=name_prefix,
                    foot_point=render_offset,
                )
                placements = atlas.placements
            write_compressed_animation_def(
                output_txt,
                image_mod_path=image_mod_path,
                sections=animation_sections,
                placements=placements,
            )
        else:
            raise RuntimeError("Grid packing txt generation is not implemented; use --packing compressed")

        print(f"Wrote {output_txt}", flush=True)

    return frame_count
