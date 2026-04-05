// ANSI sequence: {ESC}[{attr};{bg};{256colors};{fg}m

use std::io::{self, Write};

pub const DEFAULT_STYLE: &str = "\x1b[m";

const LINE_LENGTH: usize = 47;

/// Look up an ANSI escape sequence for the given style key.
/// Returns None if the key is not a valid named or numeric style.
pub fn style_for(key: &str) -> Option<String> {
    if let Some(named) = named_style(key) {
        return Some(named.to_string());
    }
    // on-N background numeric
    if let Some(num) = key.strip_prefix("on-") {
        if let Ok(i) = num.parse::<u16>() {
            if (1..=255).contains(&i) {
                return Some(format!("\x1b[48;5;{}m", i));
            }
        }
        return None;
    }
    // N foreground numeric
    if let Ok(i) = key.parse::<u16>() {
        if (1..=255).contains(&i) {
            return Some(format!("\x1b[38;5;{}m", i));
        }
    }
    None
}

fn named_style(key: &str) -> Option<&'static str> {
    Some(match key {
        "bold" => "\x1b[1m",
        "underline" => "\x1b[4m",
        "hidden" => "\x1b[4m",
        "grey" => "\x1b[30m",
        "red" => "\x1b[31m",
        "green" => "\x1b[32m",
        "yellow" => "\x1b[33m",
        "blue" => "\x1b[34m",
        "magenta" => "\x1b[35m",
        "cyan" => "\x1b[36m",
        "white" => "\x1b[37m",
        "on-grey" => "\x1b[40m",
        "on-red" => "\x1b[41m",
        "on-green" => "\x1b[42m",
        "on-yellow" => "\x1b[43m",
        "on-blue" => "\x1b[44m",
        "on-magenta" => "\x1b[45m",
        "on-cyan" => "\x1b[46m",
        "on-white" => "\x1b[47m",
        _ => return None,
    })
}

pub fn print_palette() {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    named_styles(&mut out);
    number_based_styles(&mut out);
}

fn separator<W: Write>(out: &mut W) {
    let _ = writeln!(out, " {}", "=".repeat(LINE_LENGTH));
}

fn print_style<W: Write>(out: &mut W, key: &str) {
    if let Some(s) = style_for(key) {
        let _ = write!(out, "{}{}{}", s, key, DEFAULT_STYLE);
    }
}

fn named_styles<W: Write>(out: &mut W) {
    separator(out);
    let _ = writeln!(
        out,
        "                {}Named styles{}",
        named_style("bold").unwrap(),
        DEFAULT_STYLE
    );
    separator(out);
    justify(out, &["bold", "underline"]);
    justify(out, &["grey", "red", "green", "yellow"]);
    justify(out, &["blue", "magenta", "cyan", "white"]);
    justify(out, &["on-grey", "on-red", "on-green", "on-yellow"]);
    justify(out, &["on-blue", "on-magenta", "on-cyan", "on-white"]);
}

fn number_based_styles<W: Write>(out: &mut W) {
    let _ = writeln!(out);
    separator(out);
    let _ = writeln!(
        out,
        "               {}Numeric styles{}",
        named_style("bold").unwrap(),
        DEFAULT_STYLE
    );
    separator(out);
    for i in 1..=255 {
        let _ = write!(out, "\x1b[38;5;{}m [{:>3}]", i, i);
        if i % 8 == 0 {
            let _ = writeln!(out);
        }
    }
    let _ = writeln!(out);
    separator(out);
}

fn justify<W: Write>(out: &mut W, words: &[&str]) {
    let wordcount = words.len();
    let charcount: usize = words.iter().map(|w| w.len()).sum();
    let fillsize = LINE_LENGTH - charcount;
    let spacing = fillsize / (wordcount - 1);
    let mut spacing_rem = fillsize % (wordcount - 1);

    let _ = write!(out, " "); // padding
    for word in words {
        print_style(out, word);
        let _ = write!(out, "{}", " ".repeat(spacing));
        if spacing_rem > 0 {
            let _ = write!(out, " ");
            spacing_rem -= 1;
        }
    }
    let _ = writeln!(out);
}
