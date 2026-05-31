"""Import a Mixamo FBX, fix frame range, and parent the rig to RenderPlatform."""

from __future__ import annotations

import bpy
from math import radians
from mathutils import Vector

from flare_constants import (
    CAMERA_NAME,
    DEFAULT_LOOK_HEIGHT,
    DEFAULT_ORTHO_PADDING,
    LOOK_TARGET_NAME,
    RENDER_PLATFORM_NAME,
)


def _find_armature() -> bpy.types.Object:
    armatures = [obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE"]
    if not armatures:
        raise RuntimeError("No armature found after FBX import")
    if len(armatures) > 1:
        armatures.sort(key=lambda obj: obj.name)
    return armatures[0]


def _action_has_keyframes(action: bpy.types.Action) -> bool:
    curve_range = getattr(action, "curve_frame_range", None)
    if curve_range is not None and curve_range[1] > curve_range[0]:
        return True

    fcurves = getattr(action, "fcurves", None)
    if fcurves is not None and len(fcurves) > 0:
        return True

    layers = getattr(action, "layers", None)
    if layers is not None:
        for layer in layers:
            for strip in getattr(layer, "strips", ()):
                if getattr(strip, "type", None) == "KEYFRAME":
                    return True

    return False


def _pick_action(armature: bpy.types.Object, preferred_name: str | None) -> bpy.types.Action:
    actions = list(bpy.data.actions)
    if not actions:
        raise RuntimeError("No actions found after FBX import")

    if preferred_name:
        for action in actions:
            if action.name == preferred_name or preferred_name.lower() in action.name.lower():
                return action

    for action in actions:
        if _action_has_keyframes(action):
            return action

    return actions[0]


def _clear_imported_objects() -> None:
    """Remove previously imported character meshes/armatures, keep the render rig."""
    platform = bpy.data.objects.get(RENDER_PLATFORM_NAME)
    keep = {RENDER_PLATFORM_NAME, LOOK_TARGET_NAME, CAMERA_NAME}
    keep.update(
        name
        for name in (
            "FlareKeyLight",
            "FlareFillLight",
            "FlareRimLight",
        )
    )

    to_remove = [
        obj
        for obj in bpy.context.scene.objects
        if obj.name not in keep
        and obj.type in {"ARMATURE", "MESH", "EMPTY"}
        and obj.parent != platform
    ]
    for obj in to_remove:
        bpy.data.objects.remove(obj, do_unlink=True)


def _character_objects(armature: bpy.types.Object) -> list[bpy.types.Object]:
    objects = [armature]
    for obj in armature.children_recursive:
        if obj.type in {"MESH", "ARMATURE"}:
            objects.append(obj)
    return objects


def _world_bounds(objects: list[bpy.types.Object]) -> tuple[Vector, Vector]:
    mins = Vector((1.0e9, 1.0e9, 1.0e9))
    maxs = Vector((-1.0e9, -1.0e9, -1.0e9))

    for obj in objects:
        for corner in obj.bound_box:
            world = obj.matrix_world @ Vector(corner)
            mins.x = min(mins.x, world.x)
            mins.y = min(mins.y, world.y)
            mins.z = min(mins.z, world.z)
            maxs.x = max(maxs.x, world.x)
            maxs.y = max(maxs.y, world.y)
            maxs.z = max(maxs.z, world.z)

    return mins, maxs


def _center_character_on_platform(armature: bpy.types.Object) -> tuple[Vector, Vector]:
    bpy.context.view_layer.update()
    mins, maxs = _world_bounds(_character_objects(armature))
    center = (mins + maxs) * 0.5
    armature.location.x -= center.x
    armature.location.y -= center.y
    armature.location.z -= mins.z
    bpy.context.view_layer.update()
    return _world_bounds(_character_objects(armature))


def _fit_camera_to_bounds(
    bounds: tuple[Vector, Vector],
    *,
    padding: float = DEFAULT_ORTHO_PADDING,
) -> None:
    camera = bpy.data.objects.get(CAMERA_NAME)
    look_target = bpy.data.objects.get(LOOK_TARGET_NAME)
    if camera is None or camera.type != "CAMERA":
        return

    mins, maxs = bounds
    height = maxs.z - mins.z
    width = max(maxs.x - mins.x, maxs.y - mins.y)
    span = max(height, width) * padding
    if span > 0.0:
        camera.data.ortho_scale = span

    if look_target is not None:
        look_target.location = (
            0.0,
            0.0,
            max(DEFAULT_LOOK_HEIGHT, (mins.z + maxs.z) * 0.5),
        )


def import_mixamo_fbx(
    fbx_path: str,
    *,
    animation_name: str | None = None,
    frame_start: int = 1,
    frame_count: int = 35,
    clear_scene: bool = False,
    auto_fit_camera: bool = True,
    character_z_deg: float = 0.0,
) -> bpy.types.Object:
    platform = bpy.data.objects.get(RENDER_PLATFORM_NAME)
    if platform is None:
        raise RuntimeError(
            f"Missing {RENDER_PLATFORM_NAME}. Run setup_flare_render_scene first."
        )

    if clear_scene:
        _clear_imported_objects()

    bpy.ops.import_scene.fbx(
        filepath=fbx_path,
        automatic_bone_orientation=True,
        use_anim=True,
        anim_offset=1.0,
    )

    armature = _find_armature()
    action = _pick_action(armature, animation_name)
    frame_end = frame_start + frame_count - 1

    if armature.animation_data is None:
        armature.animation_data_create()
    armature.animation_data.action = action

    action.use_frame_range = True
    action.frame_start = frame_start
    action.frame_end = frame_end

    scene = bpy.context.scene
    scene.frame_start = frame_start
    scene.frame_end = frame_end
    scene.frame_set(frame_start)

    max_dim = max(armature.dimensions)
    if max_dim > 0.0:
        target_height = 1.8
        scale_factor = target_height / max_dim
        if scale_factor < 0.5 or scale_factor > 2.0:
            armature.scale = (
                armature.scale.x * scale_factor,
                armature.scale.y * scale_factor,
                armature.scale.z * scale_factor,
            )
            bpy.context.view_layer.update()
            bpy.ops.object.select_all(action="DESELECT")
            armature.select_set(True)
            bpy.context.view_layer.objects.active = armature
            bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)

    bounds = _center_character_on_platform(armature)
    if character_z_deg:
        armature.rotation_euler[2] += radians(character_z_deg)
        bpy.context.view_layer.update()
        bounds = _world_bounds(_character_objects(armature))
    armature.parent = platform

    if auto_fit_camera:
        _fit_camera_to_bounds(bounds)

    bpy.context.view_layer.update()
    return armature
