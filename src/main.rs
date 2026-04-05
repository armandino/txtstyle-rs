use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{ArgAction, ArgGroup, Parser};

use txtstyle::confparser::ConfParser;
use txtstyle::palette;
use txtstyle::transformer::{Style, Transformer, new_regex_style};

const DEFAULT_CONF: &str = include_str!("default_conf.conf");

const VERSION_INFO: &str = concat!(
    "TxtStyle version ",
    env!("CARGO_PKG_VERSION"),
    ".\n",
    "Copyright (C) 2026 Arman Sharif.\n",
    "Apache License v2.0 or later: http://www.apache.org/licenses/LICENSE-2.0\n"
);

#[derive(Parser, Debug)]
#[command(
    name = "TxtStyle",
    about = "Prettifies output of console programs.",
    disable_version_flag = true,
    group(
        ArgGroup::new("mode")
            .args(["palette", "name", "regex"])
            .multiple(false)
    ),
)]
struct Cli {
    /// Path to a file.
    filepath: Option<PathBuf>,

    /// Print a palette of available styles.
    #[arg(short = 'p', long = "palette", action = ArgAction::SetTrue)]
    palette: bool,

    /// Name of the style to apply.
    #[arg(short = 'n', long = "name", num_args = 1)]
    name: Option<String>,

    /// Highlight text based on the given regular expression.
    #[arg(short = 'r', long = "regex", num_args = 1, action = ArgAction::Append)]
    regex: Vec<String>,

    /// Path to a conf file. Default is: ~/.txts.conf
    #[arg(short = 'c', long = "conf", num_args = 1)]
    conf: Option<PathBuf>,

    /// Always use color. Similar to grep --color=always.
    #[arg(long = "color-always", action = ArgAction::SetTrue)]
    color_always: bool,

    /// Print version information
    #[arg(long = "version", action = ArgAction::SetTrue)]
    version: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.version {
        print!("{}", VERSION_INFO);
        return ExitCode::SUCCESS;
    }
    if cli.palette {
        palette::print_palette();
        return ExitCode::SUCCESS;
    }

    let styles: Vec<Style> = if let Some(name) = &cli.name {
        let conf_path = match resolve_conf_path(cli.conf.as_deref()) {
            Ok(p) => p,
            Err(code) => return code,
        };
        let conf_lines = match read_conf_lines(&conf_path) {
            Ok(l) => l,
            Err(code) => return code,
        };
        let parser = ConfParser::new(conf_lines);
        match parser.get_styles(name) {
            Ok(s) => s,
            Err(e) => {
                let _ = writeln!(io::stderr(), "{}", e);
                return ExitCode::from(1);
            }
        }
    } else if !cli.regex.is_empty() {
        match build_regex_styles(&cli.regex) {
            Ok(s) => s,
            Err(e) => {
                let _ = writeln!(io::stderr(), "{}", e);
                return ExitCode::from(1);
            }
        }
    } else {
        Vec::new()
    };

    let transformer = Transformer::new(&styles);
    transform(&transformer, cli.filepath.as_deref(), cli.color_always)
}

fn transform(transformer: &Transformer, filepath: Option<&Path>, color_always: bool) -> ExitCode {
    if let Some(path) = filepath {
        transform_file(transformer, path, color_always)
    } else if !io::stdin().is_terminal() {
        transform_pipe(transformer, color_always)
    } else {
        ExitCode::SUCCESS
    }
}

fn transform_file(transformer: &Transformer, path: &Path, color_always: bool) -> ExitCode {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                let _ = writeln!(io::stderr(), "File not found: {}", path.display());
                return ExitCode::from(2);
            }
            let _ = writeln!(io::stderr(), "{}", e);
            return ExitCode::from(1);
        }
    };
    let reader = io::BufReader::new(file);
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let use_color = out.is_terminal() || color_always;
    for line in reader.lines() {
        match line {
            Ok(l) => {
                if write_styled_line(&mut out, transformer, &l, use_color).is_err() {
                    return ExitCode::SUCCESS; // broken pipe
                }
            }
            Err(_) => continue, // ignore decode errors (matches errors='ignore')
        }
    }
    ExitCode::SUCCESS
}

fn transform_pipe(transformer: &Transformer, color_always: bool) -> ExitCode {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let use_color = out.is_terminal() || color_always;
    for line in reader.lines() {
        match line {
            Ok(l) => {
                if write_styled_line(&mut out, transformer, &l, use_color).is_err() {
                    return ExitCode::SUCCESS;
                }
            }
            Err(_) => continue,
        }
    }
    ExitCode::SUCCESS
}

fn write_styled_line<W: Write>(
    out: &mut W,
    transformer: &Transformer,
    line: &str,
    use_color: bool,
) -> io::Result<()> {
    if use_color {
        let styled = transformer.style(line);
        writeln!(out, "{}", styled)
    } else {
        writeln!(out, "{}", line)
    }
}

fn build_regex_styles(patterns: &[String]) -> Result<Vec<Style>, String> {
    // Mirrors Python loop_default_colors(): iterates ['bold','underline'] over
    // ['red','green','blue','magenta','cyan','white'].
    let mut color_cycle: Vec<(&str, &str)> = Vec::new();
    for style in &["bold", "underline"] {
        for col in &["red", "green", "blue", "magenta", "cyan", "white"] {
            color_cycle.push((col, style));
        }
    }
    patterns
        .iter()
        .zip(color_cycle.iter().cycle())
        .map(|(pat, (col, sty))| {
            new_regex_style(pat, &[col.to_string(), sty.to_string()], false)
        })
        .collect()
}

fn resolve_conf_path(cli_conf: Option<&Path>) -> Result<PathBuf, ExitCode> {
    if let Some(p) = cli_conf {
        if !p.is_file() {
            let _ = writeln!(io::stderr(), "File not found: {}", p.display());
            return Err(ExitCode::from(2));
        }
        return Ok(p.to_path_buf());
    }
    let Some(home) = dirs::home_dir() else {
        let _ = writeln!(io::stderr(), "Cannot determine home directory");
        return Err(ExitCode::from(1));
    };
    let path = home.join(".txts.conf");
    if !path.is_file() {
        if let Err(e) = fs::write(&path, DEFAULT_CONF) {
            let _ = writeln!(io::stderr(), "Failed to create {}: {}", path.display(), e);
            return Err(ExitCode::from(1));
        }
    }
    Ok(path)
}

fn read_conf_lines(path: &Path) -> Result<Vec<String>, ExitCode> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(s.lines().map(|l| l.to_string()).collect()),
        Err(e) => {
            let _ = writeln!(io::stderr(), "{}", e);
            Err(ExitCode::from(1))
        }
    }
}
