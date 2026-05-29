use crate::error::Result;
use crate::parser::{parse_file_with_resolver, parse_str, Entry, FileResolver};
use crate::value::{parse_int, parse_point, pop_first_token, Point};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconSet {
    pub first_id: i32,
    pub path: String,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconConfig {
    pub sets: Vec<IconSet>,
    pub text_offset: Point,
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            sets: vec![IconSet {
                first_id: 0,
                path: "images/icons/icons.png".into(),
            }],
            text_offset: Point::default(),
        }
    }
}

impl IconConfig {
    pub fn parse_str(content: &str) -> Result<Self> {
        Self::from_entries(&parse_str(content, "engine/icons.txt")?)
    }

    pub fn parse_file(path: &str, resolver: &dyn FileResolver) -> Result<Self> {
        match parse_file_with_resolver(path, resolver) {
            Ok(entries) => Self::from_entries(&entries),
            Err(_) => Ok(Self::default()),
        }
    }

    pub fn from_entries(entries: &[Entry]) -> Result<Self> {
        let mut config = Self {
            sets: Vec::new(),
            text_offset: Point::default(),
        };

        for entry in entries {
            if !entry.section.is_empty() {
                continue;
            }
            match entry.key.as_str() {
                "icon_set" => {
                    let mut rest = entry.value.as_str();
                    let first_id = parse_int(&pop_first_token(&mut rest).unwrap_or_default())?;
                    let path = pop_first_token(&mut rest).unwrap_or_default();
                    config.sets.push(IconSet { first_id, path });
                }
                "text_offset" => config.text_offset = parse_point(&entry.value)?,
                _ => {}
            }
        }

        if config.sets.is_empty() {
            return Ok(Self::default());
        }

        Ok(config)
    }

    pub fn resolve_set(&self, icon_id: i32) -> Option<&IconSet> {
        self.sets
            .iter()
            .rev()
            .find(|set| icon_id >= set.first_id)
    }
}
