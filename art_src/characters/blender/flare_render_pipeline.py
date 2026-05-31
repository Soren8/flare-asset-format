#!/usr/bin/env python3
"""Headless FLARE 8-direction animation render pipeline for Blender 4.x+.

Example (``blender`` must be on ``PATH``, or set ``BLENDER`` in the shell wrapper)::

  blender --background --python flare_render_pipeline.py -- \\
    --fbx ~/Downloads/YBot_Walk.fbx \\
    --output-dir /tmp/walk_render \\
    --prefix walk \\
    --frames 35 \\
    --cell-size 128 \\
    --build-sheet /tmp/walk_render/walk.png \\
    --build-txt /tmp/walk_render/walk.txt
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import bpy

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from flare_spritesheet import assemble_flare_output  # noqa: E402
from import_mixamo import import_mixamo_fbx  # noqa: E402
from render8dirs import render_eight_directions  # noqa: E402
from setup_flare_render_scene import setup_flare_render_scene  # noqa: E402


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fbx", type=Path, help="Mixamo FBX with baked animation")
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--prefix", default="walk", help="Output filename prefix")
    parser.add_argument("--animation", default=None, help="Action name substring to pick")
    parser.add_argument("--frames", type=int, default=35, help="Number of animation frames")
    parser.add_argument("--frame-start", type=int, default=1)
    parser.add_argument("--cell-size", type=int, default=128)
    parser.add_argument("--ortho-scale", type=float, default=2.4, help="Orthographic camera scale when not auto-fitting")
    parser.add_argument(
        "--no-auto-fit-camera",
        action="store_true",
        help="Use --ortho-scale instead of fitting the camera to the imported character",
    )
    parser.add_argument(
        "--skip-render",
        action="store_true",
        help="Only build spritesheet from existing PNGs in --output-dir",
    )
    parser.add_argument("--build-sheet", type=Path, help="Optional spritesheet PNG path")
    parser.add_argument("--build-txt", type=Path, help="Optional FLARE animation definition path")
    parser.add_argument(
        "--image-mod-path",
        help="Mod-relative image path for animation txt (required with --build-txt)",
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
    parser.add_argument(
        "--render-offset",
        default="64,96",
        help="Foot point for animation.txt (pixels in cell)",
    )
    parser.add_argument(
        "--character-z-deg",
        type=float,
        default=0.0,
        help="Optional Z rotation after import to tune Mixamo facing (default: 0)",
    )
    return parser.parse_args(argv)


def _parse_render_offset(value: str) -> tuple[int, int]:
    parts = [int(part.strip()) for part in value.split(",")]
    if len(parts) != 2:
        raise SystemExit("--render-offset must be two comma-separated integers")
    return parts[0], parts[1]


def main(argv: list[str] | None = None) -> None:
    args = parse_args(argv or [])

    if not args.skip_render:
        if args.fbx is None:
            raise SystemExit("--fbx is required unless --skip-render is set")
        if not args.fbx.is_file():
            raise SystemExit(f"FBX not found: {args.fbx}")

        frame_end = args.frame_start + args.frames - 1
        setup_flare_render_scene(
            cell_size=args.cell_size,
            ortho_scale=args.ortho_scale,
            frame_start=args.frame_start,
            frame_end=frame_end,
        )
        import_mixamo_fbx(
            str(args.fbx),
            animation_name=args.animation,
            frame_start=args.frame_start,
            frame_count=args.frames,
            auto_fit_camera=not args.no_auto_fit_camera,
            character_z_deg=args.character_z_deg,
        )
        render_eight_directions(str(args.output_dir), args.prefix)

    if args.build_sheet or args.build_txt:
        if args.build_txt is not None and args.image_mod_path is None:
            raise SystemExit("--image-mod-path is required when --build-txt is set")
        assemble_flare_output(
            args.output_dir,
            name_prefix=args.prefix,
            cell_size=args.cell_size,
            output_png=args.build_sheet,
            output_txt=args.build_txt,
            image_mod_path=args.image_mod_path,
            render_offset=_parse_render_offset(args.render_offset),
            run_fps=args.run_fps,
            packing=args.packing,
        )


if __name__ == "__main__":
    argv = sys.argv
    if "--" in argv:
        argv = argv[argv.index("--") + 1 :]
    else:
        argv = []
    try:
        main(argv)
    except Exception:
        import traceback

        traceback.print_exc()
        sys.exit(1)
