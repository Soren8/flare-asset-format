//! FLARE engine movement tables and helpers (ported from flare-engine `StatBlock` / `Avatar`).

use crate::value::Direction;
use crate::{FlareError, Result};

/// Map-space delta X per direction index (SW..S CCW).
pub const DIRECTION_DELTA_X: [f32; 8] = [-1.0, -1.0, -1.0, 0.0, 1.0, 1.0, 1.0, 0.0];

/// Map-space delta Y per direction index.
pub const DIRECTION_DELTA_Y: [f32; 8] = [1.0, 0.0, -1.0, -1.0, -1.0, 0.0, 1.0, 1.0];

const SQRT_2: f32 = std::f32::consts::SQRT_2;

/// Diagonal directions move at 1/√2 speed so all facings travel one tile per cycle.
pub const SPEED_MULTIPLIER: [f32; 8] = [
    1.0 / SQRT_2,
    1.0,
    1.0 / SQRT_2,
    1.0,
    1.0 / SQRT_2,
    1.0,
    1.0 / SQRT_2,
    1.0,
];

/// Default hero walk speed in map tiles per logic frame at 60 FPS (`StatBlock::speed`).
pub const DEFAULT_BASE_SPEED: f32 = 0.1;

/// Reference logic rate used to convert real-time deltas to FLARE frame fractions.
pub const REFERENCE_FPS: f32 = 60.0;

/// Isometric keyboard → direction table from `Avatar::set_direction()` (single key = map diagonal).
pub fn direction_from_isometric_keys(
    up: bool,
    down: bool,
    left: bool,
    right: bool,
) -> Option<Direction> {
    if up && left {
        return Some(Direction::W);
    }
    if up && right {
        return Some(Direction::N);
    }
    if down && right {
        return Some(Direction::E);
    }
    if down && left {
        return Some(Direction::S);
    }
    if left {
        return Some(Direction::Sw);
    }
    if up {
        return Some(Direction::Nw);
    }
    if right {
        return Some(Direction::Ne);
    }
    if down {
        return Some(Direction::Se);
    }
    None
}

/// Movement delta for one facing, matching `Entity::move()` speed scaling.
pub fn movement_step(direction: Direction, delta_ms: f32, base_speed: f32) -> (f32, f32) {
    let idx = direction.index() as usize;
    let frame_fraction = delta_ms * REFERENCE_FPS / 1000.0;
    let speed = base_speed * SPEED_MULTIPLIER[idx] * frame_fraction;
    (
        speed * DIRECTION_DELTA_X[idx],
        speed * DIRECTION_DELTA_Y[idx],
    )
}

/// Port of `Utils::calcDirection()` for map-space vectors.
pub fn direction_from_points(x0: f32, y0: f32, x1: f32, y1: f32) -> Result<Direction> {
    let dx = x1 - x0;
    let dy = y1 - y0;
    if dx.abs() < f32::EPSILON && dy.abs() < f32::EPSILON {
        return Err(FlareError::Other("zero movement vector".into()));
    }

    let theta = calc_theta(x0, y0, x1, y1);
    let val = theta / (std::f32::consts::PI / 4.0);
    let dir = if val < 0.0 {
        (val.ceil() - 0.5) as i32
    } else {
        (val.floor() + 0.5) as i32
    } + 4;
    let dir = (dir + 1).rem_euclid(8) as u8;
    Direction::from_index(dir)
}

fn calc_theta(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dx.abs() < f32::EPSILON {
        if dy > 0.0 {
            std::f32::consts::FRAC_PI_2
        } else {
            -std::f32::consts::FRAC_PI_2
        }
    } else {
        let mut theta = (dy / dx).atan();
        if dx < 0.0 && dy >= 0.0 {
            theta += std::f32::consts::PI;
        }
        if dx < 0.0 && dy < 0.0 {
            theta -= std::f32::consts::PI;
        }
        theta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isometric_keys_match_flare_engine() {
        assert_eq!(
            direction_from_isometric_keys(true, false, false, false),
            Some(Direction::Nw)
        );
        assert_eq!(
            direction_from_isometric_keys(false, true, false, false),
            Some(Direction::Se)
        );
        assert_eq!(
            direction_from_isometric_keys(false, false, true, false),
            Some(Direction::Sw)
        );
        assert_eq!(
            direction_from_isometric_keys(false, false, false, true),
            Some(Direction::Ne)
        );
        assert_eq!(
            direction_from_isometric_keys(true, false, false, true),
            Some(Direction::N)
        );
        assert_eq!(
            direction_from_isometric_keys(true, false, true, false),
            Some(Direction::W)
        );
        assert_eq!(
            direction_from_isometric_keys(false, true, false, true),
            Some(Direction::E)
        );
        assert_eq!(
            direction_from_isometric_keys(false, true, true, false),
            Some(Direction::S)
        );
    }

    #[test]
    fn diagonal_steps_are_unit_length_per_frame() {
        let (dx, dy) = movement_step(Direction::Ne, 1000.0 / REFERENCE_FPS, DEFAULT_BASE_SPEED);
        assert!((dx * dx + dy * dy - DEFAULT_BASE_SPEED * DEFAULT_BASE_SPEED).abs() < 1e-5);
    }
}
