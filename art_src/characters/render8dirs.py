"""Legacy FLARE 8-direction render script (Blender 2.79 API).

For Blender 4.x and Mixamo FBX imports, use the automated pipeline instead:

  art_src/characters/blender/README.md
  art_src/characters/blender/run_render8dirs.sh
"""

import bpy
from math import radians

angle = 45
axis = 2  # z-axis
platform = bpy.data.objects["RenderPlatform"]
original_path = bpy.data.scenes[0].render.filepath

try:
    bpy.ops.render.view_show()
except AttributeError:
    pass

for i in range(0, 8):
    temp_rot = platform.rotation_euler
    temp_rot[axis] = temp_rot[axis] - radians(angle)
    platform.rotation_euler = temp_rot

    bpy.data.scenes[0].render.filepath = original_path + str(i)
    bpy.ops.render.render(animation=True)

bpy.data.scenes[0].render.filepath = original_path
