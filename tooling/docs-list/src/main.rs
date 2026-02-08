use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process;

const EXCLUDED_DIRS: [&str; 2] = ["archive", "research"];
const REMINDER: &str = "Reminder: keep docs up to date as behavior changes. When your task matches any \"Read when\" hint above (cache directives, database work, tests, etc.), read that doc before coding, and suggest new coverage when it is missing.";

#[derive(Debug)]
enum AppError {
    Cli { message: &'static str, code: i32 },
    Io(io::Error),
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug)]
struct Metadata {
    summary: Option<String>,
    read_when: Vec<String>,
    error: Option<&'static str>,
}

#[derive(Debug)]
enum InlineValue {
    String(String),
    Number(String),
    Bool(bool),
    Null,
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(AppError::Cli { message, code }) => {
            eprintln!("{message}");
            process::exit(code);
        }
        Err(AppError::Io(error)) if error.kind() == io::ErrorKind::BrokenPipe => {
            process::exit(0);
        }
        Err(AppError::Io(error)) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

fn run() -> Result<(), AppError> {
    let docs_dir = std::env::current_dir()?.join("docs");
    if !docs_dir.exists() {
        return Err(AppError::Cli {
            message: "docs:list: missing docs directory. Run from repo root.",
            code: 1,
        });
    }
    if !docs_dir.is_dir() {
        return Err(AppError::Cli {
            message: "docs:list: docs path is not a directory.",
            code: 1,
        });
    }

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    writeln!(out, "Listing all markdown files in docs folder:")?;
    let markdown_files = walk_markdown_files(&docs_dir, &docs_dir)?;

    for relative_path in markdown_files {
        let full_path = docs_dir.join(&relative_path);
        let metadata = extract_metadata(&full_path)?;
        if let Some(summary) = metadata.summary {
            writeln!(out, "{relative_path} - {summary}")?;
            if !metadata.read_when.is_empty() {
                writeln!(out, "  Read when: {}", metadata.read_when.join("; "))?;
            }
        } else {
            let reason = metadata
                .error
                .map(|error| format!(" - [{error}]"))
                .unwrap_or_default();
            writeln!(out, "{relative_path}{reason}")?;
        }
    }

    writeln!(out)?;
    writeln!(out, "{REMINDER}")?;
    out.flush()?;
    Ok(())
}

fn walk_markdown_files(dir: &Path, base: &Path) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }

        let full_path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if EXCLUDED_DIRS.contains(&name.as_str()) {
                continue;
            }
            files.extend(walk_markdown_files(&full_path, base)?);
        } else if file_type.is_file() && name.ends_with(".md") {
            let relative = full_path.strip_prefix(base).unwrap_or(&full_path);
            files.push(path_to_slash_string(relative));
        }
    }

    files.sort_by(|a, b| a.cmp(b));
    Ok(files)
}

fn path_to_slash_string(path: &Path) -> String {
    let mut value = path.to_string_lossy().into_owned();
    if std::path::MAIN_SEPARATOR != '/' {
        value = value.replace(std::path::MAIN_SEPARATOR, "/");
    }
    value
}

fn extract_metadata(full_path: &Path) -> io::Result<Metadata> {
    let content = fs::read_to_string(full_path)?;

    if !content.starts_with("---") {
        return Ok(Metadata {
            summary: None,
            read_when: Vec::new(),
            error: Some("missing front matter"),
        });
    }

    let end_index = match content[3..].find("\n---") {
        Some(index) => index + 3,
        None => {
            return Ok(Metadata {
                summary: None,
                read_when: Vec::new(),
                error: Some("unterminated front matter"),
            });
        }
    };

    let front_matter = content[3..end_index].trim();
    let lines = front_matter.split('\n');

    let mut summary_line: Option<String> = None;
    let mut read_when = Vec::new();
    let mut collecting_read_when = false;

    for raw_line in lines {
        let line = raw_line.trim();

        if line.starts_with("summary:") {
            summary_line = Some(line.to_string());
            collecting_read_when = false;
            continue;
        }

        if let Some(rest) = line.strip_prefix("read_when:") {
            collecting_read_when = true;
            let inline = rest.trim();
            if let Some(values) = parse_inline_read_when(inline) {
                read_when.extend(compact_strings(values));
            }
            continue;
        }

        if collecting_read_when {
            if let Some(hint) = line.strip_prefix("- ") {
                let trimmed = hint.trim();
                if !trimmed.is_empty() {
                    read_when.push(trimmed.to_string());
                }
            } else if line.is_empty() {
                continue;
            } else {
                collecting_read_when = false;
            }
        }
    }

    let Some(summary_line) = summary_line else {
        return Ok(Metadata {
            summary: None,
            read_when,
            error: Some("summary key missing"),
        });
    };

    let summary_value = summary_line["summary:".len()..].trim();
    let normalized = normalize_summary(summary_value);
    if normalized.is_empty() {
        return Ok(Metadata {
            summary: None,
            read_when,
            error: Some("summary is empty"),
        });
    }

    Ok(Metadata {
        summary: Some(normalized),
        read_when,
        error: None,
    })
}

fn normalize_summary(value: &str) -> String {
    let mut normalized = value.trim().to_string();
    if normalized.starts_with('"') || normalized.starts_with('\'') {
        normalized.remove(0);
    }
    if normalized.ends_with('"') || normalized.ends_with('\'') {
        normalized.pop();
    }
    normalized
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn compact_strings(values: Vec<InlineValue>) -> Vec<String> {
    let mut result = Vec::new();
    for value in values {
        let normalized = match value {
            InlineValue::String(value) => value.trim().to_string(),
            InlineValue::Number(value) => value.trim().to_string(),
            InlineValue::Bool(value) => value.to_string(),
            InlineValue::Null => String::new(),
        };
        if !normalized.is_empty() {
            result.push(normalized);
        }
    }
    result
}

fn parse_inline_read_when(inline: &str) -> Option<Vec<InlineValue>> {
    if !(inline.starts_with('[') && inline.ends_with(']')) {
        return None;
    }

    let inner = &inline[1..inline.len().saturating_sub(1)];
    let raw_items = split_inline_array_items(inner)?;
    let mut items = Vec::with_capacity(raw_items.len());
    for raw_item in raw_items {
        let value = parse_inline_value(raw_item.trim())?;
        items.push(value);
    }
    Some(items)
}

fn split_inline_array_items(input: &str) -> Option<Vec<String>> {
    if input.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escape = false;

    for ch in input.chars() {
        if let Some(open_quote) = quote {
            current.push(ch);
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == open_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => {
                quote = Some(ch);
                current.push(ch);
            }
            ',' => {
                items.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if quote.is_some() || escape {
        return None;
    }

    items.push(current.trim().to_string());
    if items.iter().any(|item| item.is_empty()) {
        return None;
    }
    Some(items)
}

fn parse_inline_value(value: &str) -> Option<InlineValue> {
    if value == "null" {
        return Some(InlineValue::Null);
    }
    if value == "true" {
        return Some(InlineValue::Bool(true));
    }
    if value == "false" {
        return Some(InlineValue::Bool(false));
    }

    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        if value.len() < 2 {
            return Some(InlineValue::String(String::new()));
        }
        return Some(InlineValue::String(value[1..value.len() - 1].to_string()));
    }

    if value.parse::<f64>().is_ok() {
        return Some(InlineValue::Number(value.to_string()));
    }

    None
}
