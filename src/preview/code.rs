use ratatui::{
    style::Color,
    text::{Line, Span},
};

use crate::{preview::SampleSize, theme::Theme};

pub fn lines(theme: &Theme, size: SampleSize) -> Vec<Line<'static>> {
    let sample = match size {
        SampleSize::Small => SMALL,
        SampleSize::Medium => MEDIUM,
        SampleSize::Large => LARGE,
    };
    sample.lines().map(|line| highlight(line, theme)).collect()
}

// Only the token forms used by the bundled Rust samples are needed here.
fn highlight(mut text: &'static str, theme: &Theme) -> Line<'static> {
    let normal = theme.palette.normal.map(Color::from);
    let mut spans = Vec::new();
    while let Some(first) = text.chars().next() {
        if text.starts_with("//") {
            spans.push(Span::styled(text, Color::from(theme.palette.bright[0])));
            break;
        }
        let length = if first == '"' {
            let mut escaped = false;
            text.char_indices()
                .skip(1)
                .find_map(|(offset, character)| {
                    if escaped {
                        escaped = false;
                    } else if character == '\\' {
                        escaped = true;
                    } else if character == '"' {
                        return Some(offset + 1);
                    }
                    None
                })
                .unwrap_or(text.len())
        } else if first.is_ascii_alphanumeric() || first == '_' {
            text.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(text.len())
        } else {
            first.len_utf8()
        };
        let (token, remaining) = text.split_at(length);
        let color = if first == '"' {
            Some(normal[2])
        } else if first.is_ascii_digit() {
            Some(normal[6])
        } else if matches!(
            token,
            "fn" | "let" | "mut" | "for" | "in" | "if" | "struct" | "return" | "static"
        ) {
            Some(normal[5])
        } else if first.is_ascii_alphabetic()
            && (remaining.starts_with('!') || remaining.starts_with('('))
        {
            Some(normal[4])
        } else if matches!(token, "str" | "usize") || first.is_ascii_uppercase() {
            Some(normal[3])
        } else {
            None
        };
        spans.push(match color {
            Some(color) => Span::styled(token, color),
            None => Span::raw(token),
        });
        text = remaining;
    }
    Line::from(spans)
}

const SMALL: &str = r#"fn main() {
    // Show active jobs.
    let jobs = [("build", 3)];

    for (name, count) in jobs {
        if count > 0 {
            println!("{name}: {count}");
        }
    }
}"#;

const MEDIUM: &str = r#"fn report(jobs: &[(&str, usize)]) -> usize {
    // Count work that is ready to run.
    let mut total = 0;
    for &(name, count) in jobs {
        if count > 0 {
            println!("{name}: {count}");
            total += count;
        }
    }
    total
}

fn main() {
    let jobs = [("build", 3), ("fetch", 0)];
    let total = report(&jobs);
    println!("total: {total}");
}"#;

const LARGE: &str = r#"#[derive(Debug)]
struct Job {
    name: &'static str,
    pending: usize,
}

fn report(jobs: &[Job]) -> Result<usize, &'static str> {
    // Count work that is ready to run.
    let mut total = 0;
    for job in jobs {
        if job.pending > 100 {
            return Err("queue limit exceeded");
        }
        println!("{}: {}", job.name, job.pending);
        total += job.pending;
    }
    Ok(total)
}

fn main() {
    let jobs = [Job { name: "build", pending: 3 }];
    println!("{:?}", report(&jobs));
}"#;
