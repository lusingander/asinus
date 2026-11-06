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
    let bright = theme.palette.bright.map(Color::from);
    if line.starts_with("diff --git ") {
        return Line::styled(line, bold(bright[6]));
    }
    if line.starts_with("index ") {
        return Line::styled(line, bright[0]);
    }
    if line.starts_with("@@ ") {
        return Line::styled(line, bright[5]);
    }
    if line.starts_with('-') {
        return Line::styled(line, normal[1]);
    }
    if line.starts_with('+') || line == "* main" {
        return Line::styled(line, normal[2]);
    }
    if let Some(path) = line.strip_prefix(" M") {
        return Line::from(vec![Span::styled(" M", normal[1]), Span::raw(path)]);
    }
    let (graph, rest) = line
        .strip_prefix("* ")
        .map_or((false, line), |rest| (true, rest));
    if let Some((hash, message)) = rest
        .split_once(' ')
        .filter(|(hash, _)| hash.len() == 7 && hash.bytes().all(|c| c.is_ascii_hexdigit()))
    {
        let mut spans = Vec::new();
        if graph {
            spans.push(Span::styled("* ", bright[0]));
        }
        spans.extend([Span::styled(hash, normal[3]), Span::raw(" ")]);
        if let Some(message) = message.strip_prefix("(HEAD -> main)") {
            spans.extend([
                Span::styled("(HEAD -> main)", normal[6]),
                Span::raw(message),
            ]);
        } else {
            spans.push(Span::raw(message));
        }
        return Line::from(spans);
    }
    Line::raw(line)
}

const SMALL: &str = r#"~/project $ git diff
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!("Hello, world!");
+    println!("Themes ready.");
 }

~/project $ git status --short
 M src/main.rs"#;

const MEDIUM: &str = r#"~/project $ git diff -- src/main.rs
diff --git a/src/main.rs b/src/main.rs
index 3e10ac4..6912c7d 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!("Hello, world!");
+    println!("Themes ready.");
 }

~/project $ git log --oneline --decorate -3
a1b2c3d (HEAD -> main) Add theme preview
8e4f210 Validate bundled colors
5c61b77 Add project setup

~/project $ git status --short
 M src/main.rs"#;

const LARGE: &str = r#"~/project $ git diff -- src/main.rs
diff --git a/src/main.rs b/src/main.rs
index 3e10ac4..6912c7d 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!("Hello, world!");
+    println!("Themes ready.");
 }

~/project $ git log --oneline --decorate --graph -4
* a1b2c3d (HEAD -> main) Add theme preview
* 8e4f210 Validate bundled colors
* 5c61b77 Add project setup
* 4b73019 Initial commit

~/project $ git status --short
 M src/main.rs

~/project $ git branch
* main
  feature/preview-sizes
  release/next"#;
