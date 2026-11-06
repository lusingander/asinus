mod support;

use rstest::rstest;
use support::{apply_query, quit_with as quit, resize_and_wait, screen::comparison_layout};
use termlens::{Color, Key, Screen, Terminal};

#[derive(Clone, Copy)]
struct ExpectedTheme {
    name: &'static str,
    position: usize,
}

const DAY: ExpectedTheme = ExpectedTheme {
    name: "3024 Day",
    position: 1,
};
const NIGHT: ExpectedTheme = ExpectedTheme {
    name: "3024 Night",
    position: 2,
};
const MOCHA: ExpectedTheme = ExpectedTheme {
    name: "Catppuccin Mocha",
    position: 50,
};
const LAST: ExpectedTheme = ExpectedTheme {
    name: "zenwritten_light",
    position: 348,
};
const LONG_NAME: &str = "LiquidCarbonTransparentInverse";

fn browse(
    terminal: &mut Terminal,
    theme: ExpectedTheme,
    width: u16,
    height: u16,
) -> termlens::Result<Screen> {
    browse_at(terminal, theme.name, width, height)
}

fn browse_at(
    terminal: &mut Terminal,
    name: &str,
    width: u16,
    height: u16,
) -> termlens::Result<Screen> {
    preview_at(terminal, name, "Git", width, height)
}

fn start_git(terminal: &mut Terminal, width: u16, height: u16) -> termlens::Result<Screen> {
    preview_at(terminal, DAY.name, "Palette", width, height)?;
    terminal.send_str("]]")?;
    browse(terminal, DAY, width, height)
}

fn preview_at(
    terminal: &mut Terminal,
    name: &str,
    preset: &str,
    width: u16,
    height: u16,
) -> termlens::Result<Screen> {
    terminal.wait_frame(|screen| preview_matches(screen, name, preset, width, height))
}

fn preview_matches(screen: &Screen, name: &str, preset: &str, width: u16, height: u16) -> bool {
    screen.row_text(0).contains(&format!("Preview  {preset}"))
        && screen
            .find_all(name)
            .iter()
            .any(|(row, col)| *row == 4 && *col >= 31)
        && screen.row_text(2).contains('─')
        && screen.row_text(height - 1).contains("F1 Help")
        && screen
            .cell(2, width - 2)
            .is_some_and(|cell| cell.contents() == "─")
}

#[rstest]
#[case(DAY, "Palette", "palette_minimum")]
#[case(MOCHA, "Palette", "palette_mocha_minimum")]
#[case(DAY, "Shell", "shell_minimum")]
#[case(MOCHA, "Shell", "shell_mocha_minimum")]
#[case(DAY, "Git", "git_minimum")]
#[case(MOCHA, "Git", "git_mocha_minimum")]
#[case(DAY, "Logs", "logs_minimum")]
#[case(MOCHA, "Logs", "logs_mocha_minimum")]
#[case(DAY, "TUI", "tui_minimum")]
#[case(MOCHA, "TUI", "tui_mocha_minimum")]
#[case(DAY, "Code", "code_minimum")]
#[case(MOCHA, "Code", "code_mocha_minimum")]
fn previews_match_style_snapshots(
    #[case] theme: ExpectedTheme,
    #[case] preset: &str,
    #[case] snapshot_name: &str,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(80, 24)?;
    let initial = preview_at(&mut terminal, DAY.name, "Palette", 80, 24)?;
    match preset {
        "Palette" => {}
        "Shell" => terminal.send(Key::Char(']'))?,
        "Git" => terminal.send_str("]]")?,
        "Logs" => terminal.send_str("]]]")?,
        "TUI" => terminal.send_str("]]]]")?,
        "Code" => terminal.send(Key::Char('['))?,
        _ => panic!("unexpected preset: {preset}"),
    }
    terminal.send_str(&"j".repeat(theme.position - 1))?;
    let screen = if theme.name == DAY.name && preset == "Palette" {
        initial
    } else {
        preview_at(&mut terminal, theme.name, preset, 80, 24)?
    };
    insta::assert_snapshot!(snapshot_name, screen.with_styles());
    quit(&mut terminal, Key::Char('q'))
}

#[test]
fn palette_reference_colors_match_the_hex_values() -> termlens::Result<()> {
    let mut terminal = support::isolated(100, 30)?;
    let screen = preview_at(&mut terminal, DAY.name, "Palette", 100, 30)?;
    let (row, col) = screen.find("Foreground").unwrap();
    assert_eq!(
        screen.cell(row, col).unwrap().style().fg,
        Color::Rgb(0x4a, 0x45, 0x43)
    );
    assert_eq!(
        screen.cell(20, 70).unwrap().style().bg,
        Color::Rgb(0xf7, 0xf7, 0xf7)
    );
    let (row, _) = screen
        .find_all("Red")
        .into_iter()
        .find(|(_, col)| *col == 33)
        .unwrap();
    let (_, normal) = screen.find("Normal").unwrap();
    let (_, bright) = screen.find("Bright").unwrap();
    assert_swatch(
        &screen,
        row,
        normal,
        Color::Rgb(0xdb, 0x2d, 0x20),
        "#db2d20",
    );
    assert_swatch(
        &screen,
        row,
        bright,
        Color::Rgb(0xe8, 0xbb, 0xd0),
        "#e8bbd0",
    );
    quit(&mut terminal, Key::Char('q'))
}

fn assert_swatch(screen: &Screen, row: u16, col: u16, color: Color, hex: &str) {
    assert_eq!(screen.cell(row, col).unwrap().style().bg, color);
    assert_eq!(screen.cell(row, col + 5).unwrap().style().fg, color);
    let text: String = (col + 5..col + 12)
        .map(|col| screen.cell(row, col).unwrap().contents())
        .collect();
    assert_eq!(text, hex);
}

fn assert_selection(screen: &Screen, label: &str) {
    let (y, x) = screen
        .find_all(label)
        .into_iter()
        .find(|(y, x)| *x == 5 && screen.cell(*y, *x).unwrap().style().reverse)
        .expect("selected name");
    assert_eq!(screen.cell(y, 1).unwrap().contents(), " ");
    assert_eq!(screen.cell(y, 28).unwrap().contents(), " ");
    assert_eq!(x, 5);
    assert!(screen.cell(y, 28).unwrap().style().reverse);
    assert!(!screen.cell(y, 29).unwrap().style().reverse);
    assert!(screen.cell(y, 30).unwrap().contents().trim().is_empty());
}

#[rstest]
#[case('[')]
#[case(']')]
fn presets_cycle_and_return_without_changing_selection(#[case] key: char) -> termlens::Result<()> {
    let mut t = support::isolated(80, 24)?;
    preview_at(&mut t, DAY.name, "Palette", 80, 24)?;
    let before = apply_query(&mut t, LONG_NAME, 1)?;
    assert!(preview_matches(&before, LONG_NAME, "Palette", 80, 24));
    assert_selection(&before, "LiquidCarbonTranspare..");
    for preset in if key == '[' {
        ["Code", "TUI", "Logs", "Git", "Shell", "Palette"]
    } else {
        ["Shell", "Git", "Logs", "TUI", "Code", "Palette"]
    } {
        t.send(Key::Char(key))?;
        let s = preview_at(&mut t, LONG_NAME, preset, 80, 24)?;
        assert_selection(&s, "LiquidCarbonTranspare..");
        assert!(!s.row_text(0).contains("Cycle"));
        if preset == "Palette" {
            assert_eq!(
                s.with_styles().to_string(),
                before.with_styles().to_string()
            );
        }
    }
    quit(&mut t, Key::Char('q'))
}

#[rstest]
#[case("Shell")]
#[case("Git")]
fn cli_content_is_identical_when_the_theme_changes(#[case] preset: &str) -> termlens::Result<()> {
    let mut t = support::isolated(80, 24)?;
    preview_at(&mut t, DAY.name, "Palette", 80, 24)?;
    t.send_str(if preset == "Shell" { "]" } else { "]]" })?;
    let before = preview_at(&mut t, DAY.name, preset, 80, 24)?;
    t.send_str(&"j".repeat(MOCHA.position - 1))?;
    let after = preview_at(&mut t, MOCHA.name, preset, 80, 24)?;
    for y in 6..22 {
        for x in 32..79 {
            let a = before.cell(y, x).unwrap().contents();
            let b = after.cell(y, x).unwrap().contents();
            assert_eq!(
                if a.is_empty() { " " } else { a },
                if b.is_empty() { " " } else { b }
            );
        }
    }
    assert_selection(&after, MOCHA.name);
    quit(&mut t, Key::Char('q'))
}

#[rstest]
#[case(80, 24, "Ayu Mirage", "BirdsOfParadise")]
#[case(100, 30, "Blazer", "BlulocoLight")]
fn pages_clamp_at_the_ends_and_use_the_resized_viewport(
    #[case] w: u16,
    #[case] h: u16,
    #[case] page_name: &str,
    #[case] resized: &str,
) -> termlens::Result<()> {
    let mut t = support::isolated(w, h)?;
    start_git(&mut t, w, h)?;
    t.send(Key::Char('j'))?;
    browse(&mut t, NIGHT, w, h)?;
    t.send(Key::Ctrl('f'))?;
    let page = browse_at(&mut t, page_name, w, h)?;
    assert_selection(&page, page_name);
    t.send(Key::Ctrl('b'))?;
    browse(&mut t, NIGHT, w, h)?;
    t.send(Key::Char('g'))?;
    browse(&mut t, DAY, w, h)?;
    resize_and_wait(&mut t, w, h + 6, |s| {
        preview_matches(s, DAY.name, "Git", w, h + 6)
    })?;
    t.send(Key::Ctrl('f'))?;
    browse_at(&mut t, resized, w, h + 6)?;
    t.send_str("Gj")?;
    browse(&mut t, LAST, w, h + 6)?;
    t.send_str("gk")?;
    browse(&mut t, DAY, w, h + 6)?;
    quit(&mut t, Key::Char('q'))
}

#[test]
fn pin_marks_do_not_move_or_truncate_the_name_column() -> termlens::Result<()> {
    let mut t = support::isolated(80, 24)?;
    preview_at(&mut t, DAY.name, "Palette", 80, 24)?;
    let before = apply_query(&mut t, LONG_NAME, 1)?;
    assert!(preview_matches(&before, LONG_NAME, "Palette", 80, 24));
    assert_selection(&before, "LiquidCarbonTranspare..");
    t.send(Key::Char('p'))?;
    let pinned = t.wait_frame(|s| s.row_text(1).contains("Pin"))?;
    assert_selection(&pinned, "LiquidCarbonTranspare..");
    let (y, _) = pinned
        .find_all("LiquidCarbonTranspare..")
        .into_iter()
        .find(|(y, x)| *x == 5 && pinned.cell(*y, *x).unwrap().style().reverse)
        .unwrap();
    assert_eq!(pinned.cell(y, 2).unwrap().contents(), "*");
    t.send(Key::Char('p'))?;
    let restored = t.wait_frame(|s| !s.row_text(1).contains("Pin"))?;
    assert_eq!(
        restored.with_styles().to_string(),
        before.with_styles().to_string()
    );
    quit(&mut t, Key::Ctrl('c'))
}

#[rstest]
#[case(59, 22)]
#[case(60, 21)]
fn resizing_below_the_operational_minimum_preserves_the_view(
    #[case] w: u16,
    #[case] h: u16,
    #[values("Palette", "Shell", "Git")] preset: &str,
) -> termlens::Result<()> {
    let mut t = support::isolated(80, 24)?;
    preview_at(&mut t, DAY.name, "Palette", 80, 24)?;
    if preset != "Palette" {
        t.send_str(if preset == "Shell" { "]" } else { "]]" })?;
    }
    t.send_str(&"j".repeat(MOCHA.position - 1))?;
    let before = preview_at(&mut t, MOCHA.name, preset, 80, 24)?;
    let small = resize_and_wait(&mut t, w, h, |s| {
        s.contains("Required: 60x22") && s.contains(&format!("Current:  {w}x{h}"))
    })?;
    assert!(small.contains("q Quit"));
    if w == 59 && preset == "Git" {
        insta::assert_snapshot!("terminal_too_small", small);
    }
    resize_and_wait(&mut t, 100, 30, |s| {
        preview_matches(s, MOCHA.name, preset, 100, 30)
    })?;
    let restored = resize_and_wait(&mut t, 80, 24, |s| {
        preview_matches(s, MOCHA.name, preset, 80, 24)
    })?;
    assert_eq!(
        restored.with_styles().to_string(),
        before.with_styles().to_string()
    );
    quit(&mut t, Key::Ctrl('c'))
}

#[test]
fn starts_small_and_restores_the_terminal_on_exit() -> termlens::Result<()> {
    let mut t = support::isolated(59, 22)?;
    t.wait_frame(|s| s.contains("Terminal too small"))?;
    resize_and_wait(&mut t, 60, 22, |s| {
        s.row_text(0).contains("Explore") && s.row_text(21).trim_end().ends_with("F1 Help")
    })?;
    quit(&mut t, Key::Char('q'))
}

#[rstest]
#[case(80, 24, false, "Git", "compare_horizontal_minimum")]
#[case(80, 24, true, "Git", "compare_vertical_minimum")]
#[case(134, 24, false, "Palette", "compare_palette")]
#[case(80, 40, true, "Shell", "compare_shell")]
fn comparisons_match_style_snapshots(
    #[case] w: u16,
    #[case] h: u16,
    #[case] vertical: bool,
    #[case] preset: &str,
    #[case] snapshot: &str,
) -> termlens::Result<()> {
    let mut t = support::isolated(w, h)?;
    preview_at(&mut t, DAY.name, "Palette", w, h)?;
    if preset != "Palette" {
        t.send_str(if preset == "Git" { "]]" } else { "]" })?;
    }
    t.send(Key::Char('p'))?;
    t.send_str(&"j".repeat(MOCHA.position - 1))?;
    preview_at(&mut t, MOCHA.name, preset, w, h)?;
    t.send(Key::Char('c'))?;
    if vertical {
        t.send(Key::Char('v'))?;
    }
    let screen = t.wait_frame(|s| {
        s.row_text(0).contains("Compare")
            && s.row_text(0).contains(preset)
            && comparison_layout(s, vertical)
    })?;
    insta::assert_snapshot!(snapshot, screen.with_styles());
    quit(&mut t, Key::Char('q'))
}
