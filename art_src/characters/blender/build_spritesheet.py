#!/usr/bin/env python3
"""Build a FLARE spritesheet from rendered PNG sequences (runs inside Blender).

Usage::

  blender --background --python build_spritesheet.py -- \\
    --input-dir art_src/characters/blender/render \\
    --prefix walk \\
    --output-png mods/fantasycore/images/avatar/prince/walk.png \\
    --output-txt mods/fantasycore/animations/avatar/prince/walk.txt \\
    --image-mod-path images/avatar/prince/walk.png
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from flare_constants import DEFAULT_CELL_SIZE, DEFAULT_RENDER_OFFSET  # noqa: E402
from flare_spritesheet import assemble_flare_output  # noqa: E402


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input-dir", type=Path, required=True)
    parser.add_argument("--prefix", required=True, help="Same prefix used during Blender render")
    parser.add_argument("--output-png", type=Path, help="Spritesheet PNG path")
    parser.add_argument("--output-txt", type=Path, help="FLARE animation definition path")
    parser.add_argument(
        "--image-mod-path",
        help="Mod-relative image path for the animation txt (e.g. images/avatar/prince/walk.png)",
    )
    parser.add_argument("--cell-size", type=int, default=DEFAULT_CELL_SIZE)
    parser.add_argument(
        "--render-offset",
        default=f"{DEFAULT_RENDER_OFFSET[0]},{DEFAULT_RENDER_OFFSET[1]}",
        help="Foot point in pixels, e.g. 64,96",
    )
    parser.add_argument(
        "--run-fps",
        type=float,
        default=15.0,
        help="Run playback rate in animation frames per second (duration = frames/run_fps)",
    )
    parser.add_argument(
        "--packing",
        choices=("compressed", "grid"),
        default="compressed",
        help="Spritesheet layout mode (default: compressed with tight packing)",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> None:
    args = parse_args(argv or [])
    if args.output_png is None and args.output_txt is None:
        raise SystemExit("Provide at least one of --output-png or --output-txt")
    if args.output_txt is not None and args.image_mod_path is None:
        raise SystemExit("--image-mod-path is required when --output-txt is set")

    offset_parts = [int(value.strip()) for value in args.render_offset.split(",")]
    if len(offset_parts) != 2:
        raise SystemExit("--render-offset must be two comma-separated integers")

    assemble_flare_output(
        args.input_dir,
        name_prefix=args.prefix,
        cell_size=args.cell_size,
        output_png=args.output_png,
        output_txt=args.output_txt,
        image_mod_path=args.image_mod_path,
        render_offset=(offset_parts[0], offset_parts[1]),
        run_fps=args.run_fps,
        packing=args.packing,
    )


if __name__ == "__main__":
    argv = sys.argv
    if "--" in argv:
        argv = argv[argv.index("--") + 1 :]
    else:
        argv = []
    main(argv)
