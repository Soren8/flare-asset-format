"""Create or reset the FLARE isometric render rig inside the active Blender scene."""

from __future__ import annotations

import bpy
from math import radians

from flare_constants import (
    CAMERA_NAME,
    DEFAULT_LOOK_HEIGHT,
    FILL_LIGHT_NAME,
    ISO_CAMERA_OFFSET,
    KEY_LIGHT_NAME,
    LOOK_TARGET_NAME,
    RENDER_PLATFORM_NAME,
    RIM_LIGHT_NAME,
)


def _delete_if_exists(name: str) -> None:
    obj = bpy.data.objects.get(name)
    if obj is not None:
        bpy.data.objects.remove(obj, do_unlink=True)


def _link_object(obj: bpy.types.Object) -> None:
    if obj.name not in bpy.context.scene.collection.objects:
        bpy.context.scene.collection.objects.link(obj)


def setup_render_settings(
    *,
    cell_size: int,
    frame_start: int,
    frame_end: int,
    transparent: bool = True,
) -> None:
    scene = bpy.context.scene
    scene.frame_start = frame_start
    scene.frame_end = frame_end
    scene.render.resolution_x = cell_size
    scene.render.resolution_y = cell_size
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.image_settings.color_depth = "8"
    scene.render.film_transparent = transparent
    scene.render.use_file_extension = True
    scene.render.use_overwrite = True
    scene.render.engine = "BLENDER_EEVEE"

    if scene.world is None:
        scene.world = bpy.data.worlds.new("FlareWorld")
    scene.world.use_nodes = True
    bg = scene.world.node_tree.nodes.get("Background")
    if bg is not None:
        bg.inputs[0].default_value = (0.0, 0.0, 0.0, 1.0)
        bg.inputs[1].default_value = 0.0


def _aim_camera_at_target(camera: bpy.types.Object, target: bpy.types.Object) -> None:
    for constraint in list(camera.constraints):
        camera.constraints.remove(constraint)

    camera.rotation_euler = (0.0, 0.0, 0.0)
    track = camera.constraints.new(type="TRACK_TO")
    track.target = target
    track.track_axis = "TRACK_NEGATIVE_Z"
    track.up_axis = "UP_Y"


def setup_flare_render_scene(
    *,
    cell_size: int = 128,
    ortho_scale: float = 2.4,
    frame_start: int = 1,
    frame_end: int = 35,
    look_height: float = DEFAULT_LOOK_HEIGHT,
) -> bpy.types.Object:
    """Create RenderPlatform + world-fixed camera/lights + look target on the platform.

    Only the character rig is rotated per direction (via ``RenderPlatform``).
    The camera and lights stay in world space so each facing produces a distinct view.
    """
    for name in ("Cube", "Light", "Camera"):
        _delete_if_exists(name)

    for name in (
        RENDER_PLATFORM_NAME,
        LOOK_TARGET_NAME,
        CAMERA_NAME,
        KEY_LIGHT_NAME,
        FILL_LIGHT_NAME,
        RIM_LIGHT_NAME,
    ):
        _delete_if_exists(name)

    setup_render_settings(
        cell_size=cell_size,
        frame_start=frame_start,
        frame_end=frame_end,
    )

    platform = bpy.data.objects.new(RENDER_PLATFORM_NAME, None)
    platform.empty_display_type = "PLAIN_AXES"
    platform.location = (0.0, 0.0, 0.0)
    _link_object(platform)

    look_target = bpy.data.objects.new(LOOK_TARGET_NAME, None)
    look_target.empty_display_size = 0.1
    look_target.location = (0.0, 0.0, look_height)
    _link_object(look_target)
    look_target.parent = platform

    cam_data = bpy.data.cameras.new(CAMERA_NAME)
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = ortho_scale
    cam_data.clip_start = 0.01
    cam_data.clip_end = 1000.0
    camera = bpy.data.objects.new(CAMERA_NAME, cam_data)
    camera.location = ISO_CAMERA_OFFSET
    _link_object(camera)
    _aim_camera_at_target(camera, look_target)

    scene = bpy.context.scene
    scene.camera = camera

    def add_light(
        name: str,
        energy: float,
        location: tuple[float, float, float],
        rotation: tuple[float, float, float],
    ) -> None:
        light_data = bpy.data.lights.new(name=name, type="SUN")
        light_data.energy = energy
        light_obj = bpy.data.objects.new(name, light_data)
        light_obj.location = location
        light_obj.rotation_euler = rotation
        _link_object(light_obj)

    add_light(KEY_LIGHT_NAME, 5.0, (2.0, -3.0, 6.0), (radians(50.0), radians(20.0), radians(25.0)))
    add_light(FILL_LIGHT_NAME, 2.0, (-3.0, -2.0, 4.0), (radians(60.0), radians(-25.0), radians(-35.0)))
    add_light(RIM_LIGHT_NAME, 1.5, (0.0, 4.0, 5.0), (radians(120.0), 0.0, radians(180.0)))

    bpy.context.view_layer.update()
    return platform


if __name__ == "__main__":
    setup_flare_render_scene()
