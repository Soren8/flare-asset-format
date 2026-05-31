use crate::config::TilesetConfig;
use crate::value::{Orientation, Point};

/// Convert map coordinates to screen pixel coordinates.
///
/// `cam_x`/`cam_y` are the camera position in map units. `view_w_half` and
/// `view_h_half` are half the viewport size in pixels.
pub fn map_to_screen(
    map_x: f32,
    map_y: f32,
    cam_x: f32,
    cam_y: f32,
    view_w_half: f32,
    view_h_half: f32,
    config: &TilesetConfig,
) -> Point {
    let upx = config.units_per_pixel_x();
    let upy = config.units_per_pixel_y();
    let adjust_x = (view_w_half + 0.5) * upx;
    let adjust_y = (view_h_half + 0.5) * upy;

    match config.orientation {
        Orientation::Isometric => {
            let x = (((map_x - cam_x - map_y + cam_y + adjust_x) / upx) + 0.5).floor() as i32;
            let y = (((map_x - cam_x + map_y - cam_y + adjust_y) / upy) + 0.5).floor() as i32;
            Point { x, y }
        }
        Orientation::Orthogonal => {
            let x = ((map_x - cam_x + adjust_x) / upx) as i32;
            let y = ((map_y - cam_y + adjust_y) / upy) as i32;
            Point { x, y }
        }
    }
}

/// Convert screen pixel coordinates to map coordinates.
pub fn screen_to_map(
    screen_x: i32,
    screen_y: i32,
    cam_x: f32,
    cam_y: f32,
    view_w_half: i32,
    view_h_half: i32,
    config: &TilesetConfig,
) -> (f32, f32) {
    let upx = config.units_per_pixel_x();
    let upy = config.units_per_pixel_y();

    match config.orientation {
        Orientation::Isometric => {
            let scrx = (screen_x - view_w_half) as f32 * 0.5;
            let scry = (screen_y - view_h_half) as f32 * 0.5;
            let x = upx * scrx + upy * scry + cam_x;
            let y = upy * scry - upx * scrx + cam_y;
            (x, y)
        }
        Orientation::Orthogonal => {
            let x = (screen_x - view_w_half) as f32 * upx + cam_x;
            let y = (screen_y - view_h_half) as f32 * upy + cam_y;
            (x, y)
        }
    }
}

/// Adjust a tile anchor point the way `MapRenderer::centerTile()` does.
pub fn center_tile(mut point: Point, config: &TilesetConfig) -> Point {
    match config.orientation {
        Orientation::Orthogonal => {
            point.x += config.tile_width_half();
            point.y += config.tile_height_half();
        }
        Orientation::Isometric => {
            point.y += config.tile_height_half();
        }
    }
    point
}

/// Iso entity sort key from `calculatePriosIso()` (lower draws first / behind).
pub fn iso_entity_sort_key(map_x: f32, map_y: f32) -> u64 {
    let tilex = map_x.floor() as u32;
    let tiley = map_y.floor() as u32;
    let commax = ((map_x - tilex as f32) * 1024.0) as i64;
    let commay = ((map_y - tiley as f32) * 1024.0) as i64;
    (u64::from(tilex + tiley) << 37)
        + (u64::from(tilex) << 20)
        + ((commax + commay).max(0) as u64) << 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isometric_round_trip_is_reasonable() {
        let config = TilesetConfig::default();
        let (mx, my) = screen_to_map(400, 300, 10.0, 20.0, 400, 300, &config);
        let p = map_to_screen(mx, my, 10.0, 20.0, 400.0, 300.0, &config);
        assert!((p.x - 400).abs() <= 1);
        assert!((p.y - 300).abs() <= 1);
    }
}
