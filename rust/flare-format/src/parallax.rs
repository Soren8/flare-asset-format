use crate::error::Result;
use crate::parser::{parse_file_with_resolver, parse_str, Entry, FileResolver};
use crate::value::{parse_float, parse_fpoint};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ParallaxLayer {
    pub image: String,
    pub speed: f32,
    pub fixed_speed: (f32, f32),
    pub map_layer: String,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParallaxLayers {
    pub layers: Vec<ParallaxLayer>,
}

impl ParallaxLayers {
    pub fn parse_str(content: &str) -> Result<Self> {
        Self::from_entries(&parse_str(content, "parallax.txt")?)
    }

    pub fn parse_file(path: &str, resolver: &dyn FileResolver) -> Result<Self> {
        Self::from_entries(&parse_file_with_resolver(path, resolver)?)
    }

    pub fn from_entries(entries: &[Entry]) -> Result<Self> {
        let mut layers = Vec::new();
        let mut current: Option<ParallaxLayer> = None;

        for entry in entries {
            if entry.new_section && entry.section == "layer" {
                if let Some(layer) = current.take() {
                    layers.push(layer);
                }
                current = Some(ParallaxLayer {
                    image: String::new(),
                    speed: 0.0,
                    fixed_speed: (0.0, 0.0),
                    map_layer: String::new(),
                });
                continue;
            }

            let Some(layer) = current.as_mut() else {
                continue;
            };

            match entry.key.as_str() {
                "image" => layer.image = entry.value.clone(),
                "speed" => layer.speed = parse_float(&entry.value)?,
                "fixed_speed" => layer.fixed_speed = parse_fpoint(&entry.value)?,
                "map_layer" => layer.map_layer = entry.value.clone(),
                _ => {}
            }
        }

        if let Some(layer) = current {
            layers.push(layer);
        }

        Ok(Self { layers })
    }
}
