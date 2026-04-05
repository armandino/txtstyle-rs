use crate::transformer::Style;

/// Returns a list of ((start, end), &Style) pairs describing the non-overlapping
/// styled regions of `line`. Mirrors the Python LineStyleProcessor.get_style_map
/// algorithm: styles are applied first-match-wins, with the `!` (apply_to_whole_line)
/// prefix overriding when the line is still clean.
pub fn get_style_map<'a>(line: &str, styles: &'a [Style]) -> Vec<((usize, usize), &'a Style)> {
    let mut result: Vec<((usize, usize), &'a Style)> = Vec::new();
    let mut line_is_clean = true;
    let line_length = line.len();
    let mut occupied = vec![false; line_length];

    for style in styles {
        let (mut regions, apply_to_whole_line) = match style {
            Style::Index { regions, .. } => {
                // Convert (usize, Option<usize>) -> (usize, Option<usize>)
                (regions.clone(), false)
            }
            Style::Regex {
                regex,
                apply_to_whole_line,
                ..
            } => {
                let found: Vec<(usize, Option<usize>)> = find_regions(line, regex)
                    .into_iter()
                    .map(|(s, e)| (s, Some(e)))
                    .collect();
                (found, *apply_to_whole_line)
            }
        };

        if apply_to_whole_line && !regions.is_empty() {
            if line_is_clean {
                result.push(((0, line_length), style));
                break; // can't apply any more styles
            } else {
                // skip since other styles have already been applied
                continue;
            }
        }

        // For stable processing order, iterate regions as given
        for region in regions.drain(..) {
            let (start, end_opt) = region;
            if start >= line_length {
                continue;
            }
            let end = match end_opt {
                None => line_length,
                Some(e) if e > line_length => line_length,
                Some(e) => e,
            };

            let overlaps = occupied[start..end].iter().any(|&b| b);
            if !overlaps {
                for i in start..end {
                    occupied[i] = true;
                }
                result.push(((start, end), style));
                line_is_clean = false;
            }
        }
    }

    result
}

pub fn find_regions(line: &str, regex: &regex::Regex) -> Vec<(usize, usize)> {
    if regex.as_str().is_empty() {
        return vec![];
    }
    regex.find_iter(line).map(|m| (m.start(), m.end())).collect()
}
