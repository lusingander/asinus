mod support;

use std::{fs, path::Path};

use rstest::rstest;
use support::{
    apply_query, quit, resize_and_wait,
    screen::{self, focused_field_value, matching_themes, preview_name_matches},
    wait_for,
};
use termlens::{Key, Terminal};

fn start(path: &Path) -> termlens::Result<Terminal> {
    support::start(80, 24, &path.with_extension("marks.toml"), path).map(|(terminal, _)| terminal)
}

#[test]
fn preset_selection_and_nested_help_preserve_comparison_until_apply() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let mut terminal = start(&path)?;
    terminal.send_str("pjcPl")?;
    let selected = wait_for(&mut terminal, |s| {
        focused_field_value(s, "Sample", "Shell") && s.contains("Select preview")
    })?;
    assert!(selected.row_text(0).contains("Compare"));
    assert!(selected.row_text(0).contains("Preview  Palette"));
    insta::assert_snapshot!("preset_selection_minimum", selected.with_styles());
    assert!(!path.exists());
    terminal.send_str("hh")?;
    wait_for(&mut terminal, |s| focused_field_value(s, "Sample", "Code"))?;
    terminal.send_str("ll")?;
    wait_for(&mut terminal, |s| focused_field_value(s, "Sample", "Shell"))?;
    terminal.send(Key::Tab)?;
    terminal.send(Key::Left)?;
    wait_for(&mut terminal, |s| {
        focused_field_value(s, "Content", "Large")
    })?;
    terminal.send(Key::Right)?;
    wait_for(&mut terminal, |s| focused_field_value(s, "Content", "Auto"))?;
    terminal.send_str("ll")?;
    let draft = wait_for(&mut terminal, |s| {
        focused_field_value(s, "Content", "Medium") && s.contains("Medium: 52x18 content")
    })?;
    assert!(draft.row_text(0).contains("Preview  Palette Small"));
    assert!(!path.exists());
    terminal.send(Key::F(1))?;
    wait_for(&mut terminal, |s| s.contains("Help · Select preview"))?;
    terminal.send_str("R]Sxpcq")?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        s.contains("Select preview") && focused_field_value(s, "Content", "Medium")
    })?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        !s.contains("Select preview") && s.row_text(0).contains("Compare")
    })?;
    assert!(!path.exists());
    terminal.send_str("Pl")?;
    wait_for(&mut terminal, |s| focused_field_value(s, "Sample", "Shell"))?;
    terminal.send(Key::Tab)?;
    let reset = wait_for(&mut terminal, |s| focused_field_value(s, "Content", "Auto"))?;
    assert!(reset.contains("Auto -> Small: 40x12 content"));
    terminal.send_str("lll")?;
    wait_for(&mut terminal, |s| {
        focused_field_value(s, "Content", "Large")
    })?;
    terminal.send(Key::Enter)?;
    wait_for(&mut terminal, |s| {
        !s.contains("Select preview") && s.row_text(0).contains("Preview  Shell Large")
    })?;
    let saved = fs::read_to_string(&path).unwrap();
    let value: toml::Value = toml::from_str(&saved).unwrap();
    assert_eq!(value["session"]["preset"].as_str(), Some("shell"));
    assert_eq!(value["session"]["preview_content"].as_str(), Some("large"));
    quit(&mut terminal)?;
    let mut terminal = support::spawn(80, 24, &path.with_extension("marks.toml"), &path)?;
    let restored = support::wait_for_explore(&mut terminal)?;
    assert!(restored.row_text(0).contains("Preview  Shell Large"));
    assert!(!restored.row_text(1).contains("Pin  "));
    quit(&mut terminal)
}

#[rstest]
#[case(
    "Logs",
    3,
    "worker started",
    "cause: connection timed out",
    "request=r18"
)]
#[case("TUI", 4, "Job monitor", "PROGRESS", "Details: worker")]
#[case("Code", 5, "fn main()", "fn report(jobs", "queue limit exceeded")]
fn additional_previews_share_sizes_compare_and_restore(
    #[case] preset: &str,
    #[case] steps: usize,
    #[case] common: &str,
    #[case] medium: &str,
    #[case] large: &str,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let mut terminal = start(&path)?;
    terminal.send(Key::Char('P'))?;
    terminal.send_str(&"l".repeat(steps))?;
    let draft = wait_for(&mut terminal, |s| focused_field_value(s, "Sample", preset))?;
    assert!(draft.row_text(0).contains("Preview  Palette Small"));
    terminal.send(Key::Enter)?;
    let small = wait_for(&mut terminal, |s| {
        !s.contains("Select preview") && s.row_text(0).contains(&format!("{preset} Small"))
    })?;
    assert!(small.contains(common));
    assert!(!small.contains(medium));
    let middle = resize_and_wait(&mut terminal, 88, 28, |s| {
        s.row_text(0).contains(&format!("{preset} Medium"))
    })?;
    assert!(middle.contains(medium));
    assert!(!middle.contains(large));
    let full = resize_and_wait(&mut terminal, 106, 34, |s| {
        s.row_text(0).contains(&format!("{preset} Large"))
    })?;
    assert!(full.contains(large));

    terminal.send(Key::Char('P'))?;
    terminal.send(Key::Tab)?;
    terminal.send_str("ll")?;
    wait_for(&mut terminal, |s| {
        focused_field_value(s, "Content", "Medium")
    })?;
    terminal.send(Key::Enter)?;
    wait_for(&mut terminal, |s| {
        !s.contains("Select preview") && s.row_text(0).contains(&format!("{preset} Medium"))
    })?;
    terminal.send_str("pjc")?;
    wait_for(&mut terminal, |s| {
        screen::comparison_layout(s, false)
            && preview_name_matches(s, "Pin:", "3024 Day")
            && preview_name_matches(s, "Current:", "3024 Night")
    })?;
    let horizontal = resize_and_wait(&mut terminal, 118, 30, |s| {
        screen::comparison_layout(s, false) && s.find_all(medium).len() == 2
    })?;
    assert!(horizontal.row_text(0).contains(&format!("{preset} Medium")));
    assert_eq!(horizontal.find_all(common).len(), 2);
    assert!(!horizontal.contains(large));
    terminal.send(Key::Char('v'))?;
    wait_for(&mut terminal, |s| screen::comparison_layout(s, true))?;
    let vertical = resize_and_wait(&mut terminal, 78, 65, |s| {
        screen::comparison_layout(s, true) && s.find_all(medium).len() == 2
    })?;
    assert!(vertical.row_text(0).contains(&format!("{preset} Medium")));
    assert_eq!(vertical.find_all(common).len(), 2);
    assert!(!vertical.contains(large));
    let saved = fs::read_to_string(&path).unwrap();
    let value: toml::Value = toml::from_str(&saved).unwrap();
    assert_eq!(
        value["session"]["preset"].as_str(),
        Some(preset.to_lowercase().as_str())
    );
    assert_eq!(value["session"]["preview_content"].as_str(), Some("medium"));
    quit(&mut terminal)?;

    let mut terminal = support::spawn(80, 24, &path.with_extension("marks.toml"), &path)?;
    let restored = support::wait_for_explore(&mut terminal)?;
    assert!(restored.row_text(0).contains(&format!("{preset} Medium")));
    assert_eq!(fs::read_to_string(&path).unwrap(), saved);
    quit(&mut terminal)
}

#[test]
fn help_scrolls_and_resumes_after_resize_without_changing_comparison() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let mut terminal = start(&path)?;
    terminal.send_str("pjc]")?;
    wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Compare") && s.row_text(0).contains("Preview  Shell")
    })?;
    let saved = fs::read_to_string(&path).unwrap();
    terminal.send(Key::F(1))?;
    let help = wait_for(&mut terminal, |s| s.row_text(4).contains("Help · Compare"))?;
    insta::assert_snapshot!("help_compare_minimum", help.with_styles());
    terminal.send(Key::Char('G'))?;
    wait_for(&mut terminal, |s| s.contains("Quit the application"))?;
    resize_and_wait(&mut terminal, 59, 22, |s| s.contains("Terminal too small"))?;
    resize_and_wait(&mut terminal, 120, 36, |s| {
        s.row_text(4).contains("Help · Compare") && s.contains("Quit the application")
    })?;
    terminal.send(Key::Esc)?;
    let compared = wait_for(&mut terminal, |s| {
        !s.contains("Help ·") && s.row_text(0).contains("Compare")
    })?;
    assert!(preview_name_matches(&compared, "Pin:", "3024 Day"));
    assert!(preview_name_matches(&compared, "Current:", "3024 Night"));
    assert_eq!(fs::read_to_string(&path).unwrap(), saved);
    quit(&mut terminal)
}

#[test]
fn escape_keeps_conditions_and_reset_records_initial_without_losing_pin() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let mut terminal = start(&path)?;
    apply_query(&mut terminal, "catpp type:dark", 6)?;
    terminal.send_str("pc")?;
    wait_for(&mut terminal, |s| s.row_text(0).contains("Compare"))?;
    let before = fs::read_to_string(&path).unwrap();
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Explore") && s.contains("6 / 348")
    })?;
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    terminal.send(Key::F(1))?;
    terminal.send(Key::Char('G'))?;
    wait_for(&mut terminal, |s| {
        s.row_text(4).contains("Help · Explore") && s.contains("Quit the application")
    })?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        !s.contains("Help ·") && s.contains("6 / 348")
    })?;
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    terminal.send_str("cR")?;
    let reset = wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Compare") && matching_themes(s, 348)
    })?;
    assert!(preview_name_matches(&reset, "Pin:", "Catppuccin Frappe"));
    let value: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(value["session"]["query"].as_str(), Some("ignore:false"));
    quit(&mut terminal)?;
    let mut terminal = support::spawn(80, 24, &path.with_extension("marks.toml"), &path)?;
    let restored = support::wait_for_explore(&mut terminal)?;
    assert!(restored.contains("348 themes"));
    assert!(!restored.row_text(1).contains("Pin  "));
    quit(&mut terminal)
}

#[test]
fn preview_content_resizes_automatically_and_fixed_sizes_keep_the_same_comparison_content()
-> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let mut terminal = start(&path)?;
    terminal.send(Key::Char(']'))?;
    let small = wait_for(&mut terminal, |s| s.row_text(0).contains("Shell Small"))?;
    assert!(small.contains("Summary: 2 passed, 1 failed"));
    resize_and_wait(&mut terminal, 87, 28, |s| {
        s.row_text(0).contains("Shell Small")
    })?;
    resize_and_wait(&mut terminal, 88, 27, |s| {
        s.row_text(0).contains("Shell Small")
    })?;
    resize_and_wait(&mut terminal, 88, 28, |s| {
        s.row_text(0).contains("Shell Medium") && s.contains("Finished in 0.42s")
    })?;
    resize_and_wait(&mut terminal, 105, 34, |s| {
        s.row_text(0).contains("Shell Medium")
    })?;
    resize_and_wait(&mut terminal, 106, 33, |s| {
        s.row_text(0).contains("Shell Medium")
    })?;
    resize_and_wait(&mut terminal, 106, 34, |s| {
        s.row_text(0).contains("Shell Large") && s.contains("tree -L 2 --noreport src")
    })?;
    let saved = fs::read_to_string(&path).unwrap();
    assert!(saved.contains("preview_content = \"auto\""));
    resize_and_wait(&mut terminal, 80, 24, |s| {
        s.row_text(0).contains("Shell Small")
    })?;
    assert_eq!(fs::read_to_string(&path).unwrap(), saved);
    terminal.send_str("pjc")?;
    let horizontal = wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Compare") && s.row_text(0).contains("Shell Small")
    })?;
    assert_eq!(horizontal.find_all("Summary: 2 passed, 1 failed").len(), 2);
    terminal.send(Key::Char('v'))?;
    let vertical = wait_for(&mut terminal, |s| screen::comparison_layout(s, true))?;
    assert_eq!(vertical.find_all("~/project $ ls -F").len(), 2);
    assert!(!vertical.contains("Summary:"));
    let tall = resize_and_wait(&mut terminal, 78, 65, |s| {
        s.row_text(0).contains("Shell Large")
    })?;
    assert_eq!(tall.find_all("tree -L 2 --noreport src").len(), 2);
    terminal.send(Key::Char('P'))?;
    terminal.send(Key::Tab)?;
    terminal.send_str("lll")?;
    wait_for(&mut terminal, |s| {
        focused_field_value(s, "Content", "Large")
    })?;
    terminal.send(Key::Enter)?;
    wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Shell Large") && !s.contains("Select preview")
    })?;
    let compact = resize_and_wait(&mut terminal, 60, 22, |s| {
        s.row_text(0).contains("Shell Large")
    })?;
    assert_eq!(compact.find_all("~/project $ ls -lh").len(), 2);
    assert!(!compact.contains("Finished in 0.42s"));
    terminal.send(Key::Char('P'))?;
    terminal.send(Key::Tab)?;
    wait_for(&mut terminal, |s| {
        focused_field_value(s, "Content", "Large") && s.contains("Large: 70x24 content")
    })?;
    terminal.send(Key::BackTab)?;
    wait_for(&mut terminal, |s| focused_field_value(s, "Sample", "Shell"))?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| !s.contains("Select preview"))?;
    resize_and_wait(&mut terminal, 78, 65, |s| {
        s.row_text(0).contains("Shell Large") && s.find_all("tree -L 2 --noreport src").len() == 2
    })?;
    terminal.send(Key::Esc)?;
    let explore = wait_for(&mut terminal, |s| s.row_text(0).contains("Explore"))?;
    assert!(explore.row_text(0).contains("Shell Large"));
    terminal.send(Key::Char('['))?;
    wait_for(&mut terminal, |s| s.row_text(0).contains("Palette Large"))?;
    quit(&mut terminal)
}
