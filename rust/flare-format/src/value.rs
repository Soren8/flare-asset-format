use crate::error::{FlareError, Result};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum Direction {
    #[default]
    Sw = 0,
    W = 1,
    Nw = 2,
    N = 3,
    Ne = 4,
    E = 5,
    Se = 6,
    S = 7,
}

impl Direction {
    pub fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(Self::Sw),
            1 => Ok(Self::W),
            2 => Ok(Self::Nw),
            3 => Ok(Self::N),
            4 => Ok(Self::Ne),
            5 => Ok(Self::E),
            6 => Ok(Self::Se),
            7 => Ok(Self::S),
            _ => Err(FlareError::Other(format!(
                "direction index {index} is not within range 0-7"
            ))),
        }
    }

    pub fn index(self) -> u8 {
        self as u8
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    Isometric,
    Orthogonal,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Add,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationType {
    #[default]
    PlayOnce,
    Looped,
    BackForth,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveSubFrame {
    #[default]
    End,
    Start,
    All,
}

pub fn trim(s: &str) -> &str {
    s.trim()
}

pub fn skip_line(line: &str) -> bool {
    let line = trim(line);
    line.is_empty() || line.starts_with('#')
}

pub fn get_section_title(line: &str) -> String {
    let line = trim(line);
    let inner = line.trim_start_matches('[').trim_end_matches(']');
    inner.to_string()
}

pub fn split_key_value(line: &str) -> Option<(String, String)> {
    let line = trim(line);
    let pos = line.find('=')?;
    let key = trim(&line[..pos]).to_string();
    let val = trim(&line[pos + 1..]).to_string();
    Some((key, val))
}

pub fn pop_first_token(val: &mut &str) -> Option<String> {
    let val_trimmed = trim(val);
    if val_trimmed.is_empty() {
        *val = "";
        return None;
    }

    let sep = if val_trimmed.contains(',') {
        ','
    } else if val_trimmed.contains(';') {
        ';'
    } else {
        let token = val_trimmed.to_string();
        *val = "";
        return Some(token);
    };

    let (token, rest) = val_trimmed.split_once(sep)?;
    *val = rest;
    Some(trim(token).to_string())
}

pub fn parse_int(s: &str) -> Result<i32> {
    trim(s)
        .parse()
        .map_err(|_| FlareError::Other(format!("invalid integer: {s}")))
}

pub fn parse_float(s: &str) -> Result<f32> {
    trim(s)
        .parse()
        .map_err(|_| FlareError::Other(format!("invalid float: {s}")))
}

pub fn parse_bool(s: &str) -> bool {
    matches!(trim(s).to_lowercase().as_str(), "true" | "1" | "yes")
}

pub fn parse_point(s: &str) -> Result<Point> {
    let mut rest = s;
    let x = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other(format!("expected point x,y, got: {s}"))
    })?)?;
    let y = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other(format!("expected point x,y, got: {s}"))
    })?)?;
    Ok(Point { x, y })
}

pub fn parse_fpoint(s: &str) -> Result<(f32, f32)> {
    let mut rest = s;
    let x = parse_float(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other(format!("expected float point x,y, got: {s}"))
    })?)?;
    let y = parse_float(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other(format!("expected float point x,y, got: {s}"))
    })?)?;
    Ok((x, y))
}

pub fn parse_color_rgb(s: &str) -> Result<Color> {
    let mut rest = s;
    let r = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other(format!("expected color r,g,b, got: {s}"))
    })?)? as u8;
    let g = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other(format!("expected color r,g,b, got: {s}"))
    })?)? as u8;
    let b = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other(format!("expected color r,g,b, got: {s}"))
    })?)? as u8;
    Ok(Color {
        r,
        g,
        b,
        a: 255,
    })
}

pub fn parse_color_rgba(s: &str) -> Result<Color> {
    let mut rest = s;
    let mut color = parse_color_rgb(s)?;
    if let Some(a) = pop_first_token(&mut rest) {
        color.a = parse_int(&a)? as u8;
    }
    Ok(color)
}

/// Parse a duration string and return milliseconds.
///
/// Supports `{N}ms` and `{N}s`. Values without a suffix are treated as milliseconds.
pub fn parse_duration_ms(s: &str) -> Result<u32> {
    let s = trim(s);
    if s.is_empty() {
        return Ok(0);
    }

    if let Some(num) = s.strip_suffix("ms") {
        return Ok(parse_int(num)?.max(0) as u32);
    }
    if let Some(num) = s.strip_suffix('s') {
        let seconds = parse_int(num)?.max(0) as u32;
        return Ok(seconds.saturating_mul(1000));
    }

    Ok(parse_int(s)?.max(0) as u32)
}

pub fn parse_direction(s: &str) -> Result<Direction> {
    match trim(s) {
        "N" => Ok(Direction::N),
        "NE" => Ok(Direction::Ne),
        "E" => Ok(Direction::E),
        "SE" => Ok(Direction::Se),
        "S" => Ok(Direction::S),
        "SW" => Ok(Direction::Sw),
        "W" => Ok(Direction::W),
        "NW" => Ok(Direction::Nw),
        other => Direction::from_index(parse_int(other)? as u8),
    }
}

pub fn parse_blend_mode(s: &str) -> Result<BlendMode> {
    match trim(s) {
        "normal" => Ok(BlendMode::Normal),
        "add" => Ok(BlendMode::Add),
        other => Err(FlareError::Other(format!("invalid blend mode: {other}"))),
    }
}

pub fn parse_animation_type(s: &str) -> Result<AnimationType> {
    match trim(s) {
        "play_once" => Ok(AnimationType::PlayOnce),
        "looped" => Ok(AnimationType::Looped),
        "back_forth" => Ok(AnimationType::BackForth),
        other => Err(FlareError::Other(format!("invalid animation type: {other}"))),
    }
}

pub fn parse_active_sub_frame(s: &str) -> Result<ActiveSubFrame> {
    match trim(s) {
        "end" => Ok(ActiveSubFrame::End),
        "start" => Ok(ActiveSubFrame::Start),
        "all" => Ok(ActiveSubFrame::All),
        other => Err(FlareError::Other(format!(
            "invalid active_sub_frame: {other}"
        ))),
    }
}

pub fn parse_orientation(s: &str) -> Result<Orientation> {
    match trim(s) {
        "isometric" => Ok(Orientation::Isometric),
        "orthogonal" => Ok(Orientation::Orthogonal),
        other => Err(FlareError::Other(format!("invalid orientation: {other}"))),
    }
}
