use regex::Regex;

use crate::linestyle::get_style_map;
use crate::palette::{DEFAULT_STYLE, style_for};

#[derive(Debug)]
pub enum Style {
    Regex {
        regex: Regex,
        pattern: String,
        transforms: String,
        apply_to_whole_line: bool,
    },
    Index {
        regions: Vec<(usize, Option<usize>)>,
        transforms: String,
    },
}

impl Style {
    pub fn transforms(&self) -> &str {
        match self {
            Style::Regex { transforms, .. } => transforms,
            Style::Index { transforms, .. } => transforms,
        }
    }

    pub fn pattern(&self) -> Option<&str> {
        match self {
            Style::Regex { pattern, .. } => Some(pattern),
            _ => None,
        }
    }

    pub fn apply_to_whole_line(&self) -> bool {
        match self {
            Style::Regex {
                apply_to_whole_line,
                ..
            } => *apply_to_whole_line,
            _ => false,
        }
    }

    pub fn regions(&self) -> Option<&[(usize, Option<usize>)]> {
        match self {
            Style::Index { regions, .. } => Some(regions),
            _ => None,
        }
    }
}

/// Build a transforms string from a list of style keys. Returns an error
/// message matching the Python exception text if any key is invalid.
pub fn build_transforms(keys: &[String]) -> Result<String, String> {
    let mut out = String::new();
    for k in keys {
        match style_for(k) {
            Some(s) => out.push_str(&s),
            None => return Err(format!("Invalid style key: \"{}\"", k)),
        }
    }
    Ok(out)
}

pub fn new_regex_style(
    pattern: &str,
    transform_keys: &[String],
    apply_to_whole_line: bool,
) -> Result<Style, String> {
    let transforms = build_transforms(transform_keys)?;
    let regex = Regex::new(pattern).map_err(|e| format!("Invalid regex: {}", e))?;
    Ok(Style::Regex {
        regex,
        pattern: pattern.to_string(),
        transforms,
        apply_to_whole_line,
    })
}

pub fn new_index_style(
    regions: Vec<(usize, Option<usize>)>,
    transform_keys: &[String],
) -> Result<Style, String> {
    let transforms = build_transforms(transform_keys)?;
    Ok(Style::Index {
        regions,
        transforms,
    })
}

pub struct Transformer<'a> {
    styles: &'a [Style],
}

impl<'a> Transformer<'a> {
    pub fn new(styles: &'a [Style]) -> Self {
        Self { styles }
    }

    pub fn style(&self, line: &str) -> String {
        if self.styles.is_empty() {
            return line.to_string();
        }

        let mut style_map = get_style_map(line, self.styles);
        style_map.sort_by_key(|(r, _)| *r);

        let mut pos: usize = 0;
        let mut styled_line = String::new();
        let line_len = line.len();

        for ((start, end), style) in &style_map {
            let (start, end) = (*start, *end);

            if pos < start {
                append_to(&mut styled_line, line, pos, start, None);
            }
            append_to(&mut styled_line, line, start, end, Some(style.transforms()));
            pos = end;
        }

        // Python: `if pos <= len(line) - 1` (equivalent to pos < line_len)
        if line_len > 0 && pos < line_len {
            append_to(&mut styled_line, line, pos, line_len, None);
        }

        styled_line
    }
}

fn append_to(out: &mut String, line: &str, start: usize, end: usize, style: Option<&str>) {
    if let Some(s) = style {
        out.push_str(s);
    }
    out.push_str(&line[start..end]);
    out.push_str(DEFAULT_STYLE);
}
