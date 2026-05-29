use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{FlareError, Result};
use crate::value::{get_section_title, skip_line, split_key_value, trim};

/// A single key/value entry from a FLARE definition file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub section: String,
    pub key: String,
    pub value: String,
    pub line: usize,
    pub new_section: bool,
}

/// Trait for resolving mod-relative file paths.
pub trait FileResolver {
    fn resolve(&self, path: &str) -> Result<Option<PathBuf>>;
    fn resolve_all(&self, path: &str) -> Result<Vec<PathBuf>>;
}

/// Filesystem resolver with mod roots in ascending priority (later roots override earlier ones).
#[derive(Debug, Clone, Default)]
pub struct FsResolver {
    pub roots: Vec<PathBuf>,
}

impl FsResolver {
    pub fn new(roots: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            roots: roots.into_iter().collect(),
        }
    }

    pub fn push_root(&mut self, root: PathBuf) {
        self.roots.push(root);
    }
}

impl FileResolver for FsResolver {
    fn resolve(&self, path: &str) -> Result<Option<PathBuf>> {
        let rel = normalize_path(path);
        for root in self.roots.iter().rev() {
            let candidate = root.join(&rel);
            if candidate.is_file() {
                return Ok(Some(candidate));
            }
        }
        Ok(None)
    }

    fn resolve_all(&self, path: &str) -> Result<Vec<PathBuf>> {
        let rel = normalize_path(path);
        let mut found = Vec::new();
        for root in &self.roots {
            let candidate = root.join(&rel);
            if candidate.is_file() {
                found.push(candidate);
            }
        }
        Ok(found)
    }
}

fn normalize_path(path: &str) -> PathBuf {
    Path::new(path.replace('\\', "/").as_str()).to_path_buf()
}

fn file_starts_with_append(path: &Path) -> Result<bool> {
    let content = fs::read_to_string(path)?;
    for line in content.lines() {
        let line = trim(line);
        if skip_line(line) {
            continue;
        }
        return Ok(line == "APPEND");
    }
    Ok(false)
}

fn select_base_index(paths: &[PathBuf]) -> Result<usize> {
    for (idx, path) in paths.iter().enumerate().rev() {
        if !file_starts_with_append(path)? {
            return Ok(idx);
        }
    }
    Ok(0)
}

/// Parse a FLARE definition file from a string.
pub fn parse_str(content: &str, source: &str) -> Result<Vec<Entry>> {
    parse_str_with_include(content, source, &NullResolver)
}

struct NullResolver;

impl FileResolver for NullResolver {
    fn resolve(&self, path: &str) -> Result<Option<PathBuf>> {
        Err(FlareError::FileNotFound(path.to_string()))
    }

    fn resolve_all(&self, _path: &str) -> Result<Vec<PathBuf>> {
        Ok(Vec::new())
    }
}

/// Parse a mod-relative file using a resolver (handles APPEND/INCLUDE across mod roots).
pub fn parse_file(path: &str, resolver: &dyn FileResolver) -> Result<Vec<Entry>> {
    parse_file_with_resolver(path, resolver)
}

/// Parse a file with an explicit resolver that also handles INCLUDE directives.
pub fn parse_file_with_resolver(path: &str, resolver: &dyn FileResolver) -> Result<Vec<Entry>> {
    let paths = resolver.resolve_all(path)?;
    if paths.is_empty() {
        return Err(FlareError::FileNotFound(path.to_string()));
    }

    let base_index = select_base_index(&paths)?;
    let mut all_entries = Vec::new();

    for file_path in &paths[base_index..] {
        let content = fs::read_to_string(file_path)?;
        let entries = parse_str_with_include(&content, path, resolver)?;
        all_entries.extend(entries);
    }

    Ok(all_entries)
}

fn parse_str_with_include(
    content: &str,
    source: &str,
    resolver: &dyn FileResolver,
) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    let mut current_section = String::new();

    for (line_number, line) in content.lines().enumerate() {
        let line_number = line_number + 1;
        if skip_line(line) {
            continue;
        }

        if trim(line) == "APPEND" {
            continue;
        }

        if trim(line).starts_with('[') {
            current_section = get_section_title(line);
            entries.push(Entry {
                section: current_section.clone(),
                key: String::new(),
                value: String::new(),
                line: line_number,
                new_section: true,
            });
            continue;
        }

        if let Some(rest) = trim(line).strip_prefix("INCLUDE ") {
            let include_path = trim(rest).to_string();
            if include_path == source {
                return Err(FlareError::RecursiveInclude(include_path));
            }
            let resolved = resolver.resolve(&include_path)?.ok_or_else(|| {
                FlareError::FileNotFound(include_path.clone())
            })?;
            let included = fs::read_to_string(resolved)?;
            let mut nested = parse_str_with_include(&included, &include_path, resolver)?;
            for entry in &mut nested {
                if entry.section.is_empty() {
                    entry.section = current_section.clone();
                }
            }
            entries.append(&mut nested);
            continue;
        }

        let (key, value) =
            split_key_value(line).ok_or_else(|| FlareError::parse(source, line_number, "expected key=value"))?;
        entries.push(Entry {
            section: current_section.clone(),
            key,
            value,
            line: line_number,
            new_section: false,
        });
    }

    Ok(entries)
}

/// Collect raw lines following a `data=` key (used by map layers).
pub fn collect_data_rows(content: &str, data_line: usize) -> Result<Vec<String>> {
    let lines: Vec<&str> = content.lines().collect();
    if data_line >= lines.len() {
        return Ok(Vec::new());
    }

    let mut rows = Vec::new();
    for line in lines.iter().skip(data_line + 1) {
        if skip_line(line) {
            continue;
        }
        if trim(line).starts_with('[') || trim(line).contains('=') {
            break;
        }
        rows.push(trim(line).trim_end_matches(',').to_string());
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sections_and_keys() {
        let content = r#"
# comment
image=images/test.png

[run]
frames=4
duration=800ms
"#;
        let entries = parse_str(content, "test.txt").unwrap();
        assert!(entries.iter().any(|e| e.key == "image"));
        assert!(entries.iter().any(|e| e.section == "run" && e.key == "frames"));
    }
}
