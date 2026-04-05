use regex::Regex;

use crate::transformer::{Style, new_index_style, new_regex_style};

pub struct ConfParser {
    lines: Vec<String>,
}

impl ConfParser {
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }

    pub fn get_styles(&self, style_name: &str) -> Result<Vec<Style>, String> {
        let defs = self.get_style_defs(style_name)?;
        defs.iter().map(|d| parse_style(d)).collect()
    }

    fn get_style_defs(&self, style_name: &str) -> Result<Vec<String>, String> {
        let header_re = style_header_regex();
        let mut defs: Vec<String> = Vec::new();
        let mut in_style = false;

        for raw in &self.lines {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if is_header_matching(&header_re, line, Some(style_name)) {
                in_style = true;
            } else if in_style && is_header_matching(&header_re, line, None) {
                break;
            } else if in_style {
                defs.push(line.to_string());
            }
        }

        if !in_style {
            return Err(format!("Style \"{}\" is not defined", style_name));
        }
        Ok(defs)
    }
}

fn style_header_regex() -> Regex {
    // ^\[\s*Style\s*=\s*"?(\w+)"?\s*\]$
    Regex::new(r#"^\[\s*Style\s*=\s*"?(\w+)"?\s*\]$"#).unwrap()
}

fn is_header_matching(re: &Regex, line: &str, name: Option<&str>) -> bool {
    match re.captures(line) {
        None => false,
        Some(caps) => match name {
            None => true,
            Some(n) => caps.get(1).map(|m| m.as_str() == n).unwrap_or(false),
        },
    }
}

fn parse_style(style_def: &str) -> Result<Style, String> {
    if let Some(style) = try_parse_regex(style_def)? {
        return Ok(style);
    }
    if let Some(style) = try_parse_index(style_def)? {
        return Ok(style);
    }
    Err(format!("Invalid style definition: {}", style_def))
}

fn regex_style_regex() -> Regex {
    // ^(!?)([\w|\s|-]+):\s*regex\(['|"](.+)['|"]\)$
    Regex::new(r#"^(!?)([\w|\s|-]+):\s*regex\(['|"](.+)['|"]\)$"#).unwrap()
}

fn index_style_regex() -> Regex {
    // ^([\w|\s|-]+):\s*index\(\s*(.+)\s*\)$
    Regex::new(r"^([\w|\s|-]+):\s*index\(\s*(.+)\s*\)$").unwrap()
}

fn try_parse_regex(style_def: &str) -> Result<Option<Style>, String> {
    let re = regex_style_regex();
    let Some(caps) = re.captures(style_def) else {
        return Ok(None);
    };
    let apply_to_whole_line = caps.get(1).unwrap().as_str().trim() == "!";
    let transforms: Vec<String> = caps
        .get(2)
        .unwrap()
        .as_str()
        .trim()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    let pattern = caps.get(3).unwrap().as_str().trim().to_string();
    Ok(Some(new_regex_style(
        &pattern,
        &transforms,
        apply_to_whole_line,
    )?))
}

fn try_parse_index(style_def: &str) -> Result<Option<Style>, String> {
    let re = index_style_regex();
    let Some(caps) = re.captures(style_def) else {
        return Ok(None);
    };
    let transforms: Vec<String> = caps
        .get(1)
        .unwrap()
        .as_str()
        .trim()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    let regionlist = caps.get(2).unwrap().as_str().trim();
    let mut regions: Vec<(usize, Option<usize>)> = Vec::new();

    for item in regionlist.split(',') {
        let parts: Vec<&str> = item.split('-').collect();
        if parts.is_empty() {
            return Err(format!("Invalid style definition: {}", style_def));
        }
        let start: usize = parts[0]
            .trim()
            .parse()
            .map_err(|_| format!("Invalid style definition: {}", style_def))?;
        let end: Option<usize> = if parts.len() >= 2 {
            let s = parts[1].trim();
            if s.is_empty() {
                None
            } else {
                Some(
                    s.parse::<usize>()
                        .map_err(|_| format!("Invalid style definition: {}", style_def))?,
                )
            }
        } else {
            None
        };
        if let Some(e) = end {
            if start >= e {
                return Err(format!(
                    "Invalid style definition: {} (Start index [{}] >= end index [{}])",
                    style_def, start, e
                ));
            }
        }
        regions.push((start, end));
    }

    if regions.is_empty() {
        return Err(format!("Invalid style definition: {}", style_def));
    }

    Ok(Some(new_index_style(regions, &transforms)?))
}
