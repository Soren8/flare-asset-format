use std::collections::HashMap;

use crate::error::{FlareError, Result};
use crate::parser::{parse_file_with_resolver, parse_str, Entry, FileResolver};
use crate::value::{
    parse_active_sub_frame, parse_animation_type, parse_blend_mode, parse_color_rgb,
    parse_direction, parse_duration_ms, parse_int, parse_point, pop_first_token, ActiveSubFrame,
    AnimationType, BlendMode, Color, Direction, Point, Rect,
};

pub const DIRECTIONS: usize = 8;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub index: u16,
    pub direction: Direction,
    pub src: Rect,
    pub offset: Point,
    pub image_id: String,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Animation {
    pub name: String,
    pub animation_type: AnimationType,
    pub frames: u16,
    pub duration_ms: u32,
    pub position: u16,
    pub blend_mode: BlendMode,
    pub alpha_mod: u8,
    pub color_mod: Color,
    pub active_frames: Vec<i16>,
    pub active_sub_frame: ActiveSubFrame,
    pub compressed: bool,
    pub frame_data: Vec<Frame>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationSet {
    pub images: HashMap<String, String>,
    pub image_order: Vec<String>,
    pub render_size: Option<Point>,
    pub render_offset: Point,
    pub blend_mode: BlendMode,
    pub alpha_mod: u8,
    pub color_mod: Color,
    pub animations: Vec<Animation>,
    pub default_animation: String,
}

impl AnimationSet {
    pub fn parse_str(content: &str) -> Result<Self> {
        Self::from_entries(&parse_str(content, "animation.txt")?)
    }

    pub fn parse_file(path: &str, resolver: &dyn FileResolver) -> Result<Self> {
        Self::from_entries(&parse_file_with_resolver(path, resolver)?)
    }

    pub fn from_entries(entries: &[Entry]) -> Result<Self> {
        let mut set = Self {
            images: HashMap::new(),
            image_order: Vec::new(),
            render_size: None,
            render_offset: Point::default(),
            blend_mode: BlendMode::Normal,
            alpha_mod: 255,
            color_mod: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            animations: Vec::new(),
            default_animation: String::new(),
        };

        let mut current_section = String::new();
        let mut section_state = SectionState::default();
        let mut first_section = true;

        for entry in entries {
            if entry.new_section && !entry.section.is_empty() {
                if !first_section {
                    set.finish_section(&current_section, &mut section_state)?;
                }
                first_section = false;
                current_section = entry.section.clone();
                section_state = SectionState::default();
                if set.default_animation.is_empty() {
                    set.default_animation = current_section.clone();
                }
                continue;
            }

            if entry.section.is_empty() {
                match entry.key.as_str() {
                    "image" => {
                        let mut rest = entry.value.as_str();
                        let path = pop_first_token(&mut rest).unwrap_or_default();
                        let id = pop_first_token(&mut rest).unwrap_or_default();
                        let key = if id.is_empty() { path.clone() } else { id };
                        if !set.images.contains_key(&key) {
                            set.image_order.push(key.clone());
                        }
                        set.images.insert(key, path);
                    }
                    "render_size" => set.render_size = Some(parse_point(&entry.value)?),
                    "render_offset" => set.render_offset = parse_point(&entry.value)?,
                    "blend_mode" => set.blend_mode = parse_blend_mode(&entry.value)?,
                    "alpha_mod" => set.alpha_mod = parse_int(&entry.value)? as u8,
                    "color_mod" => set.color_mod = parse_color_rgb(&entry.value)?,
                    _ => {}
                }
            } else {
                current_section = entry.section.clone();
                match entry.key.as_str() {
                    "position" => {
                        section_state.position = parse_int(&entry.value)? as u16;
                    }
                    "frames" => section_state.frames = parse_int(&entry.value)? as u16,
                    "duration" => section_state.duration_ms = parse_duration_ms(&entry.value)?,
                    "type" => section_state.animation_type = Some(parse_animation_type(&entry.value)?),
                    "active_frame" => {
                        section_state.active_frames.clear();
                        let mut rest = entry.value.as_str();
                        let first = pop_first_token(&mut rest).unwrap_or_default();
                        if first == "all" {
                            section_state.active_all = true;
                        } else {
                            section_state
                                .active_frames
                                .push(parse_int(&first)? as i16);
                            while let Some(token) = pop_first_token(&mut rest) {
                                section_state
                                    .active_frames
                                    .push(parse_int(&token)? as i16);
                            }
                        }
                    }
                    "active_sub_frame" => {
                        section_state.active_sub_frame = parse_active_sub_frame(&entry.value)?;
                    }
                    "frame" => {
                        section_state.compressed = true;
                        section_state.frame_lines.push(entry.value.clone());
                    }
                    _ => {}
                }
            }
        }

        if !current_section.is_empty() {
            set.finish_section(&current_section, &mut section_state)?;
        }

        if set.default_animation.is_empty() && !set.animations.is_empty() {
            set.default_animation = set.animations[0].name.clone();
        }

        Ok(set)
    }

    fn finish_section(
        &mut self,
        section: &str,
        state: &mut SectionState,
    ) -> Result<()> {
        let animation_type = state
            .animation_type
            .unwrap_or(AnimationType::PlayOnce);

        let mut active_frames = state.active_frames.clone();
        if state.active_all {
            active_frames = vec![-1];
        }

        if state.compressed {
            let mut frame_data = Vec::new();
            for line in &state.frame_lines {
                frame_data.push(parse_frame_line(line, self)?);
            }
            self.animations.push(Animation {
                name: section.to_string(),
                animation_type,
                frames: state.frames,
                duration_ms: state.duration_ms,
                position: state.position,
                blend_mode: self.blend_mode,
                alpha_mod: self.alpha_mod,
                color_mod: self.color_mod,
                active_frames,
                active_sub_frame: state.active_sub_frame,
                compressed: true,
                frame_data,
            });
        } else {
            let render_size = self.render_size.ok_or_else(|| {
                FlareError::Other(format!(
                    "uncompressed animation '{section}' requires global render_size"
                ))
            })?;
            let mut frame_data = Vec::new();
            for frame_index in 0..state.frames {
                for direction in 0..DIRECTIONS {
                    let dir = Direction::from_index(direction as u8)?;
                    frame_data.push(Frame {
                        index: frame_index,
                        direction: dir,
                        src: Rect {
                            x: render_size.x * (state.position as i32 + frame_index as i32),
                            y: render_size.y * direction as i32,
                            w: render_size.x,
                            h: render_size.y,
                        },
                        offset: self.render_offset,
                        image_id: self
                            .image_order
                            .first()
                            .cloned()
                            .unwrap_or_default(),
                    });
                }
            }
            self.animations.push(Animation {
                name: section.to_string(),
                animation_type,
                frames: state.frames,
                duration_ms: state.duration_ms,
                position: state.position,
                blend_mode: self.blend_mode,
                alpha_mod: self.alpha_mod,
                color_mod: self.color_mod,
                active_frames,
                active_sub_frame: state.active_sub_frame,
                compressed: false,
                frame_data,
            });
        }

        Ok(())
    }

    pub fn animation(&self, name: &str) -> Option<&Animation> {
        self.animations.iter().find(|a| a.name == name)
    }

    pub fn default_animation(&self) -> Option<&Animation> {
        self.animation(&self.default_animation)
            .or_else(|| self.animations.first())
    }

    pub fn image_path(&self, image_id: &str) -> Option<&str> {
        if image_id.is_empty() {
            return self
                .image_order
                .first()
                .and_then(|id| self.images.get(id))
                .map(String::as_str);
        }
        self.images.get(image_id).map(String::as_str)
    }
}

#[derive(Default)]
struct SectionState {
    position: u16,
    frames: u16,
    duration_ms: u32,
    animation_type: Option<AnimationType>,
    active_frames: Vec<i16>,
    active_all: bool,
    active_sub_frame: ActiveSubFrame,
    compressed: bool,
    frame_lines: Vec<String>,
}

fn parse_frame_line(line: &str, _set: &AnimationSet) -> Result<Frame> {
    let mut rest = line;
    let index = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other("frame line missing index".into())
    })?)? as u16;
    let direction = parse_direction(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other("frame line missing direction".into())
    })?)?;
    let x = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other("frame line missing x".into())
    })?)?;
    let y = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other("frame line missing y".into())
    })?)?;
    let w = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other("frame line missing width".into())
    })?)?;
    let h = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other("frame line missing height".into())
    })?)?;
    let offset_x = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other("frame line missing offset_x".into())
    })?)?;
    let offset_y = parse_int(&pop_first_token(&mut rest).ok_or_else(|| {
        FlareError::Other("frame line missing offset_y".into())
    })?)?;
    let image_id = pop_first_token(&mut rest).unwrap_or_default();

    Ok(Frame {
        index,
        direction,
        src: Rect { x, y, w, h },
        offset: Point {
            x: offset_x,
            y: offset_y,
        },
        image_id,
    })
}

impl Animation {
    pub fn frame(&self, index: u16, direction: Direction) -> Option<&Frame> {
        self.frame_data
            .iter()
            .find(|f| f.index == index && f.direction == direction)
    }
}

/// Runtime animation playback state (milliseconds-based).
#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    animation: Animation,
    elapsed_ms: f32,
    frame_index: u16,
    reverse: bool,
    finished: bool,
    loop_count: u32,
    active_triggered: bool,
}

impl AnimationPlayer {
    pub fn new(animation: Animation) -> Self {
        Self {
            animation,
            elapsed_ms: 0.0,
            frame_index: 0,
            reverse: false,
            finished: false,
            loop_count: 0,
            active_triggered: false,
        }
    }

    pub fn reset(&mut self) {
        self.elapsed_ms = 0.0;
        self.frame_index = 0;
        self.reverse = false;
        self.finished = false;
        self.loop_count = 0;
        self.active_triggered = false;
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn loop_count(&self) -> u32 {
        self.loop_count
    }

    pub fn frame_index(&self) -> u16 {
        self.frame_index
    }

    pub fn animation(&self) -> &Animation {
        &self.animation
    }

    pub fn current_frame(&self, direction: Direction) -> Option<&Frame> {
        self.animation.frame(self.frame_index, direction)
    }

    pub fn advance(&mut self, dt_ms: f32) -> AnimationEvent {
        if self.finished || self.animation.frames == 0 || self.animation.duration_ms == 0 {
            return AnimationEvent::None;
        }

        let total_ms = match self.animation.animation_type {
            AnimationType::BackForth => self.animation.duration_ms as f32 * 2.0,
            _ => self.animation.duration_ms as f32,
        };

        let frame_ms = total_ms / self.animation.frames as f32;
        let prev_index = self.frame_index;
        self.elapsed_ms += dt_ms;

        let mut event = AnimationEvent::None;

        loop {
            if self.elapsed_ms < frame_ms {
                break;
            }
            self.elapsed_ms -= frame_ms;

            match self.animation.animation_type {
                AnimationType::PlayOnce => {
                    if self.frame_index + 1 >= self.animation.frames {
                        self.frame_index = self.animation.frames.saturating_sub(1);
                        self.finished = true;
                        break;
                    }
                    self.frame_index += 1;
                }
                AnimationType::Looped => {
                    self.frame_index = (self.frame_index + 1) % self.animation.frames;
                    if self.frame_index == 0 {
                        self.loop_count += 1;
                        event = AnimationEvent::Looped;
                    }
                }
                AnimationType::BackForth => {
                    if !self.reverse {
                        if self.frame_index + 1 >= self.animation.frames {
                            if self.animation.frames <= 1 {
                                self.loop_count += 1;
                                event = AnimationEvent::Looped;
                            }
                            self.reverse = true;
                        } else {
                            self.frame_index += 1;
                        }
                    } else if self.frame_index == 0 {
                        self.reverse = false;
                        self.loop_count += 1;
                        event = AnimationEvent::Looped;
                    } else {
                        self.frame_index -= 1;
                    }
                }
            }
        }

        if prev_index != self.frame_index && self.is_active_frame(self.frame_index) {
            self.active_triggered = true;
            return AnimationEvent::ActiveFrame {
                frame: self.frame_index,
            };
        }

        event
    }

    fn is_active_frame(&self, frame: u16) -> bool {
        if self.animation.active_frames.is_empty() {
            return false;
        }
        if self.animation.active_frames.len() == 1 && self.animation.active_frames[0] == -1 {
            return true;
        }
        self.animation
            .active_frames
            .iter()
            .any(|&f| f as u16 == frame)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationEvent {
    None,
    Looped,
    ActiveFrame { frame: u16 },
}
