use crate::error::{FlareError, Result};
use crate::parser::FileResolver;
use crate::value::{
    parse_bool, parse_color_rgba, parse_int, parse_point, pop_first_token, skip_line, split_key_value,
    trim, Color,
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct MapHeader {
    pub title: String,
    pub width: u16,
    pub height: u16,
    pub tileset: String,
    pub music: String,
    pub hero_pos: Option<(f32, f32)>,
    pub parallax_layers: String,
    pub background_color: Color,
    pub fogofwar: u16,
    pub save_fogofwar: bool,
}

impl Default for MapHeader {
    fn default() -> Self {
        Self {
            title: String::new(),
            width: 1,
            height: 1,
            tileset: String::new(),
            music: String::new(),
            hero_pos: None,
            parallax_layers: String::new(),
            background_color: Color::default(),
            fogofwar: 0,
            save_fogofwar: false,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapLayer {
    pub name: String,
    pub data: Vec<Vec<u16>>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSectionEntry {
    pub key: String,
    pub value: String,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSection {
    pub name: String,
    pub entries: Vec<RawSectionEntry>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TilesetExportEntry {
    pub path: String,
    pub tile_width: i32,
    pub tile_height: i32,
    pub offset_x: i32,
    pub offset_y: i32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    pub header: MapHeader,
    pub tileset_exports: Vec<TilesetExportEntry>,
    pub layers: Vec<MapLayer>,
    pub raw_sections: Vec<RawSection>,
}

impl Map {
    pub fn parse_str(content: &str) -> Result<Self> {
        Self::from_content(content, "map.txt")
    }

    pub fn parse_file(path: &str, resolver: &dyn FileResolver) -> Result<Self> {
        let paths = resolver.resolve_all(path)?;
        let content = if let Some(file) = paths.last() {
            std::fs::read_to_string(file)?
        } else {
            return Err(FlareError::FileNotFound(path.to_string()));
        };
        Self::from_content(&content, path)
    }

    pub fn from_content(content: &str, source: &str) -> Result<Self> {
        let lines: Vec<&str> = content.lines().collect();
        let mut map = Self {
            header: MapHeader::default(),
            tileset_exports: Vec::new(),
            layers: Vec::new(),
            raw_sections: Vec::new(),
        };

        let mut section = String::new();
        let mut current_layer: Option<MapLayer> = None;
        let mut current_raw: Option<RawSection> = None;
        let mut reading_layer_data = false;
        let mut layer_rows_read = 0usize;

        let mut i = 0usize;
        while i < lines.len() {
            let line_number = i + 1;
            let line = lines[i];
            i += 1;

            if skip_line(line) {
                continue;
            }

            let trimmed = trim(line);
            if trimmed.starts_with('[') {
                if let Some(layer) = current_layer.take() {
                    map.layers.push(layer);
                }
                if let Some(raw) = current_raw.take() {
                    map.raw_sections.push(raw);
                }
                reading_layer_data = false;
                section = crate::value::get_section_title(trimmed);
                if matches!(section.as_str(), "enemy" | "npc" | "event") {
                    current_raw = Some(RawSection {
                        name: section.clone(),
                        entries: Vec::new(),
                    });
                }
                continue;
            }

            if reading_layer_data {
                if trimmed.contains('=') {
                    reading_layer_data = false;
                    i -= 1;
                    continue;
                }
                let layer = current_layer.as_mut().ok_or_else(|| {
                    FlareError::parse(source, line_number, "layer data without active layer")
                })?;
                layer
                    .data
                    .push(parse_layer_row(trimmed.trim_end_matches(','), map.header.width)?);
                layer_rows_read += 1;
                if layer_rows_read >= map.header.height as usize {
                    reading_layer_data = false;
                }
                continue;
            }

            if trimmed == "APPEND" {
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("INCLUDE ") {
                let include_path = trim(rest);
                let included = std::fs::read_to_string(include_path).map_err(|_| {
                    FlareError::FileNotFound(include_path.to_string())
                })?;
                let nested = Self::from_content(&included, include_path)?;
                map.merge(nested);
                continue;
            }

            let (key, value) = split_key_value(trimmed)
                .ok_or_else(|| FlareError::parse(source, line_number, "expected key=value"))?;

            match section.as_str() {
                "header" => apply_header(&mut map.header, &key, &value)?,
                "tilesets" => {
                    let mut rest = value.as_str();
                    let path = pop_first_token(&mut rest).unwrap_or_default();
                    let tile_width = parse_int(&pop_first_token(&mut rest).unwrap_or_default())?;
                    let tile_height = parse_int(&pop_first_token(&mut rest).unwrap_or_default())?;
                    let offset_x = parse_int(&pop_first_token(&mut rest).unwrap_or_default())?;
                    let offset_y = parse_int(&pop_first_token(&mut rest).unwrap_or_default())?;
                    map.tileset_exports.push(TilesetExportEntry {
                        path,
                        tile_width,
                        tile_height,
                        offset_x,
                        offset_y,
                    });
                }
                "layer" => match key.as_str() {
                    "type" => {
                        if let Some(layer) = current_layer.take() {
                            map.layers.push(layer);
                        }
                        current_layer = Some(MapLayer {
                            name: value.clone(),
                            data: Vec::new(),
                        });
                    }
                    "format" if value != "dec" => {
                        return Err(FlareError::parse(
                            source,
                            line_number,
                            "layer format must be 'dec'",
                        ));
                    }
                    "data" => {
                        let _ = current_layer.as_mut().ok_or_else(|| {
                            FlareError::parse(source, line_number, "data before layer type")
                        })?;
                        reading_layer_data = true;
                        layer_rows_read = 0;
                    }
                    _ => {}
                },
                "enemy" | "npc" | "event" => {
                    let raw = current_raw.as_mut().ok_or_else(|| {
                        FlareError::parse(source, line_number, "raw section missing")
                    })?;
                    raw.entries.push(RawSectionEntry { key, value });
                }
                _ => {}
            }
        }

        if let Some(layer) = current_layer {
            map.layers.push(layer);
        }
        if let Some(raw) = current_raw {
            map.raw_sections.push(raw);
        }

        ensure_collision_layer(&mut map);
        Ok(map)
    }

    fn merge(&mut self, other: Self) {
        if !other.header.tileset.is_empty() {
            self.header = other.header;
        }
        self.tileset_exports.extend(other.tileset_exports);
        self.layers.extend(other.layers);
        self.raw_sections.extend(other.raw_sections);
    }

    pub fn layer(&self, name: &str) -> Option<&MapLayer> {
        self.layers.iter().find(|l| l.name == name)
    }

    pub fn collision_layer(&self) -> Option<&MapLayer> {
        self.layer("collision")
    }
}

fn apply_header(header: &mut MapHeader, key: &str, value: &str) -> Result<()> {
    match key {
        "title" => header.title = value.to_string(),
        "width" => header.width = parse_int(value)?.max(1) as u16,
        "height" => header.height = parse_int(value)?.max(1) as u16,
        "tileset" => header.tileset = value.to_string(),
        "music" => header.music = value.to_string(),
        "hero_pos" => {
            let pos = parse_point(value)?;
            header.hero_pos = Some((pos.x as f32 + 0.5, pos.y as f32 + 0.5));
        }
        "parallax_layers" => header.parallax_layers = value.to_string(),
        "background_color" => header.background_color = parse_color_rgba(value)?,
        "fogofwar" => header.fogofwar = parse_int(value)? as u16,
        "save_fogofwar" => header.save_fogofwar = parse_bool(value),
        "tilewidth" | "tileheight" | "orientation" => {}
        _ => {}
    }
    Ok(())
}

fn parse_layer_row(row: &str, width: u16) -> Result<Vec<u16>> {
    let mut rest = row;
    let mut values = Vec::new();
    while let Some(token) = pop_first_token(&mut rest) {
        if token.is_empty() {
            continue;
        }
        values.push(parse_int(&token)? as u16);
    }
    if values.len() != width as usize {
        return Err(FlareError::Other(format!(
            "layer row width {} != map width {}",
            values.len(),
            width
        )));
    }
    Ok(values)
}

fn ensure_collision_layer(map: &mut Map) {
    if map.layer("collision").is_some() {
        return;
    }
    let width = map.header.width as usize;
    let height = map.header.height as usize;
    let data = vec![vec![0u16; height]; width];
    map.layers.push(MapLayer {
        name: "collision".into(),
        data,
    });
}
