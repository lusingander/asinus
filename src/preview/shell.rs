use ratatui::{
    style::Color,
    text::{Line, Span},
};

use crate::{
    preview::{
        SampleSize,
        style::{bold, prompt},
    },
    theme::Theme,
};

pub fn lines(theme: &Theme, size: SampleSize) -> Vec<Line<'static>> {
    let sample = match size {
        SampleSize::Small => SMALL,
        SampleSize::Medium => MEDIUM,
        SampleSize::Large => LARGE,
    };
    sample.lines().map(|line| highlight(line, theme)).collect()
}

fn highlight(line: &'static str, theme: &Theme) -> Line<'static> {
    if let Some(command) = line.strip_prefix("~/project $ ") {
        return prompt(theme, command);
    }
    let normal = theme.palette.normal.map(Color::from);
    for (label, index) in [("[info]", 6), ("[warn]", 3), ("[pass]", 2), ("[fail]", 1)] {
        if line.starts_with(label) {
            return Line::styled(line, normal[index]);
        }
    }
    if line.starts_with("Summary:") {
        return Line::styled(line, bold(normal[1]));
    }
    if line.starts_with("total ") || line.starts_with("Finished ") {
        return Line::styled(line, Color::from(theme.palette.bright[0]));
    }
    let mut spans = Vec::new();
    for part in line.split_inclusive(' ') {
        let name = part.trim_end_matches(' ');
        let color = match name {
            "src/" | "tests/" | "build/" => Some(normal[4]),
            "check.sh" | "check.sh*" => Some(normal[2]),
            "latest" | "latest@" => Some(normal[6]),
            _ => None,
        };
        if let Some(color) = color {
            spans.push(Span::styled(name, bold(color)));
            spans.push(Span::raw(&part[name.len()..]));
        } else {
            spans.push(Span::raw(part));
        }
    }
    Line::from(spans)
}

const SMALL: &str = "\
~/project $ ls -F
src/  tests/
Cargo.toml  check.sh*
latest@

~/project $ ./check.sh
[info] 3 checks
[warn] cache is empty
[pass] config syntax
[pass] input validation
[fail] missing sample.txt
Summary: 2 passed, 1 failed";

const MEDIUM: &str = "\
~/project $ ls -lh
total 16K
drwxr-xr-x  6 dev staff 192 Oct 12 src/
drwxr-xr-x  4 dev staff 128 Oct 12 tests/
-rw-r--r--  1 dev staff 418 Oct 12 Cargo.toml
-rwxr-xr-x  1 dev staff 256 Oct 12 check.sh
lrwxr-xr-x  1 dev staff   5 Oct 12 latest -> build/

~/project $ ./check.sh
[info] running 3 checks
[warn] cache is empty; using defaults
[pass] config syntax
[pass] input validation
[fail] missing fixture: sample.txt

Summary: 2 passed, 1 failed
Finished in 0.42s
~/project $ ";

const LARGE: &str = "\
~/project $ ls -lh
total 16K
drwxr-xr-x  6 dev staff 192 Oct 12 src/
drwxr-xr-x  4 dev staff 128 Oct 12 tests/
-rw-r--r--  1 dev staff 418 Oct 12 Cargo.toml
-rwxr-xr-x  1 dev staff 256 Oct 12 check.sh
lrwxr-xr-x  1 dev staff   5 Oct 12 latest -> build/

~/project $ ./check.sh
[info] running 3 checks
[warn] cache is empty; using defaults
[pass] config syntax
[pass] input validation
[fail] missing fixture: sample.txt

Summary: 2 passed, 1 failed
Finished in 0.42s

~/project $ tree -L 2 --noreport src
src/
|-- main.rs
|-- preview.rs
`-- ui.rs
~/project $ ";
