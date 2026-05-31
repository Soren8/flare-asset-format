"""Shared constants for FLARE isometric 8-direction rendering."""

from math import radians, sqrt

# FLARE engine direction order (CCW from SW). render8dirs uses index 0..7 as row index.
FLARE_DIRECTIONS = ("SW", "W", "NW", "N", "NE", "E", "SE", "S")

# Rotate RenderPlatform -Z each step (same as legacy render8dirs.py).
DIRECTION_STEP_DEG = 45.0
DIRECTION_AXIS = 2  # Z

RENDER_PLATFORM_NAME = "RenderPlatform"
LOOK_TARGET_NAME = "FlareLookTarget"
CAMERA_NAME = "FlareCamera"
KEY_LIGHT_NAME = "FlareKeyLight"
FILL_LIGHT_NAME = "FlareFillLight"
RIM_LIGHT_NAME = "FlareRimLight"

# Isometric camera sits in the +X, -Y, +Z octant and looks at the origin.
# Equal offsets give ~35.26 deg elevation (standard isometric).
ISO_CAMERA_OFFSET = (8.0, -8.0, 8.0)
ISO_CAMERA_DISTANCE = sqrt(
    ISO_CAMERA_OFFSET[0] ** 2 + ISO_CAMERA_OFFSET[1] ** 2 + ISO_CAMERA_OFFSET[2] ** 2
)

DEFAULT_CELL_SIZE = 128
DEFAULT_RENDER_OFFSET = (64, 96)  # typical humanoid foot point for 128x128 cells
DEFAULT_LOOK_HEIGHT = 0.5
DEFAULT_ORTHO_PADDING = 1.15

# Legacy constants kept for reference; the camera now uses Track To instead.
CAMERA_ROTATION_X = radians(60.0)
CAMERA_ROTATION_Z = radians(45.0)
CAMERA_LOCATION = (0.0, -7.5, 7.5)
