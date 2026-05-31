"""Render the active animation in eight FLARE directions.

Modern replacement for the legacy ``../render8dirs.py`` script.
Rotate ``RenderPlatform`` by 45° per direction (same algorithm as upstream FLARE).
"""

from __future__ import annotations

import os
from math import radians

import bpy

from flare_constants import DIRECTION_AXIS, DIRECTION_STEP_DEG, FLARE_DIRECTIONS, RENDER_PLATFORM_NAME


def render_eight_directions(
    output_dir: str,
    name_prefix: str,
    *,
    directions: int = 8,
) -> list[str]:
    platform = bpy.data.objects.get(RENDER_PLATFORM_NAME)
    if platform is None:
        raise RuntimeError(f"Object {RENDER_PLATFORM_NAME} not found")

    os.makedirs(output_dir, exist_ok=True)

    scene = bpy.context.scene
    original_path = scene.render.filepath
    base_prefix = os.path.join(output_dir, name_prefix)

    # Reset platform rotation so repeated runs are deterministic.
    platform.rotation_euler[0] = 0.0
    platform.rotation_euler[1] = 0.0
    platform.rotation_euler[2] = 0.0

    written_dirs: list[str] = []

    for direction_index in range(directions):
        # Match legacy FLARE render8dirs.py: rotate platform before each render.
        rot = platform.rotation_euler
        rot[DIRECTION_AXIS] = rot[DIRECTION_AXIS] - radians(DIRECTION_STEP_DEG)
        platform.rotation_euler = rot

        direction_label = FLARE_DIRECTIONS[direction_index] if direction_index < len(FLARE_DIRECTIONS) else str(direction_index)
        filepath_prefix = f"{base_prefix}{direction_index}_"
        scene.render.filepath = filepath_prefix

        print(
            f"Rendering direction {direction_index} ({direction_label}) -> {filepath_prefix}####.png",
            flush=True,
        )
        bpy.ops.render.render(animation=True, write_still=False)
        written_dirs.append(filepath_prefix)

    scene.render.filepath = original_path
    return written_dirs


if __name__ == "__main__":
    render_eight_directions("/tmp/flare_render", "walk")
