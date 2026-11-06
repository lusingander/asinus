#![cfg(any(target_os = "macos", target_os = "linux"))]

mod support;

use std::{fs, os::unix::fs::PermissionsExt, path::Path};

use rstest::rstest;
use serde_json::Value;
use support::{
    quit, resize_and_wait,
    screen::{
        comparison_layout, footer_contains, is_selected, shows_current_theme, theme_list_visible,
    },
};
use termlens::{Key, Screen, Terminal};

fn install_commands(path: &Path) {
    for name in ["pbcopy", "wl-copy", "xclip"] {
        let command = path.join(name);
        fs::write(
            &command,
            r##"#!/bin/sh
/bin/cat > "$COPY_CAPTURE"
if [ -f "$COPY_FAIL" ]; then
    echo "clipboard access denied" >&2
    exit 9
fi
"##,
        )
        .unwrap();
        fs::set_permissions(command, fs::Permissions::from_mode(0o700)).unwrap();
    }
}

fn start(path: &Path, width: u16, height: u16) -> termlens::Result<Terminal> {
    install_commands(path);
    let mut terminal = termlens::bin!(
        "asinus",
        size(width, height),
        env("ASINUS_STATE_PATH", path.join("marks.toml")),
        env("ASINUS_WORKSPACE_PATH", path.join("workspace.toml")),
        env("PATH", path),
        env("WAYLAND_DISPLAY", "test-wayland"),
        env("DISPLAY", ":test"),
        env("COPY_CAPTURE", path.join("copied.txt")),
        env("COPY_FAIL", path.join("fail"))
    )?;
    support::wait_for_explore(&mut terminal)?;
    Ok(terminal)
}

fn body(screen: &Screen) -> Vec<String> {
    (1..22).map(|row| screen.row_text(row)).collect()
}

#[rstest]
#[case("", "Explore", true)]
#[case("z", "Explore", false)]
#[case("c", "Compare", false)]
#[case("cv", "Compare", false)]
#[case("cz", "Compare", true)]
#[case("czv", "Compare", true)]
fn copying_both_targets_and_formats_keeps_the_exploration_and_saved_preferences(
    #[case] keys: &str,
    #[case] mode: &str,
    #[case] list_visible: bool,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();
    let mut terminal = start(path, 80, 24)?;
    terminal.send_str("pj]")?;
    terminal.send_str(keys)?;
    let before = terminal.wait_frame(|s| {
        s.row_text(0).contains(mode)
            && s.row_text(0).contains("Preview  Shell")
            && shows_current_theme(s, "3024 Night")
            && theme_list_visible(s) == list_visible
            && (mode != "Compare" || comparison_layout(s, keys.ends_with('v')))
    })?;
    let saved = fs::read(path.join("workspace.toml")).unwrap();
    terminal.send(Key::Char('y'))?;
    terminal.wait_frame(|s| {
        s.contains("Copy theme") && s.contains("Target: Current") && is_selected(s, "Name")
    })?;
    terminal.send_str("pzxRc/S]yq")?;
    terminal.send(Key::Tab)?;
    terminal.send(Key::BackTab)?;
    terminal.wait_frame(|s| s.contains("Target: Current") && is_selected(s, "Name"))?;
    terminal.send(Key::Char('t'))?;
    terminal.wait_frame(|s| s.contains("Target: Pin") && s.contains("Theme: 3024 Day"))?;
    terminal.send(Key::Char('t'))?;
    terminal.wait_frame(|s| s.contains("Target: Current") && s.contains("Theme: 3024 Night"))?;
    terminal.send(Key::F(1))?;
    terminal.wait_frame(|s| s.row_text(4).contains("Help · Copy theme"))?;
    terminal.send(Key::Char('t'))?;
    terminal.send(Key::Enter)?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| s.contains("Copy theme") && s.contains("Target: Current"))?;
    resize_and_wait(&mut terminal, 59, 22, |s| s.contains("Current:  59x22"))?;
    resize_and_wait(&mut terminal, 120, 36, |s| {
        s.contains("Target: Current")
            && is_selected(s, "Name")
            && s.contains("Enter Copy")
            && footer_contains(s, "F1 Help")
    })?;
    resize_and_wait(&mut terminal, 80, 24, |s| {
        s.contains("Target: Current")
            && s.contains("Enter Copy")
            && s.row_text(0).contains(mode)
            && footer_contains(s, "F1 Help")
    })?;
    terminal.send(Key::Esc)?;
    let restored = terminal
        .wait_frame(|s| !s.contains("Copy theme") && s.row_text(0).contains("Preview  Shell"))?;
    assert_eq!(body(&restored), body(&before));
    assert!(!path.join("copied.txt").exists());
    terminal.send(Key::Char('y'))?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| footer_contains(s, "Copied Name: 3024 Night"))?;
    assert_eq!(
        fs::read_to_string(path.join("copied.txt")).unwrap(),
        "3024 Night"
    );
    terminal.send_str("yj")?;
    terminal.send(Key::Char('t'))?;
    terminal.wait_frame(|s| s.contains("Target: Pin") && is_selected(s, "JSON"))?;
    terminal.send(Key::Enter)?;
    let copied = terminal.wait_frame(|s| footer_contains(s, "Copied JSON: 3024 Day"))?;
    assert_eq!(body(&copied), body(&before));
    let text = fs::read_to_string(path.join("copied.txt")).unwrap();
    assert!(!text.ends_with('\n'));
    let json: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["background"], "#f7f7f7");
    assert_eq!(json["cursor"], "#4a4543");
    assert_eq!(json["selection"], "#a5a2a2");
    assert_eq!(json.as_object().unwrap().len(), 20);
    assert_eq!(fs::read(path.join("workspace.toml")).unwrap(), saved);
    assert!(!path.join("marks.toml").exists());
    quit(&mut terminal)
}

#[test]
fn copy_failure_preserves_the_selection_and_retries_the_same_content() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();
    let mut terminal = start(path, 80, 24)?;
    terminal.send_str("pjzcyj")?;
    terminal.send(Key::Char('t'))?;
    terminal.wait_frame(|s| s.contains("Target: Pin") && is_selected(s, "JSON"))?;
    fs::write(path.join("fail"), "").unwrap();
    terminal.send(Key::Enter)?;
    let failed = terminal
        .wait_frame(|s| footer_contains(s, "Error: Copy failed:") && s.contains("Enter Copy"))?;
    assert!(failed.contains("Theme: 3024 Day"));
    assert!(is_selected(&failed, "JSON"));
    let payload = fs::read(path.join("copied.txt")).unwrap();
    fs::remove_file(path.join("fail")).unwrap();
    terminal.send(Key::Enter)?;
    let copied = terminal
        .wait_frame(|s| !s.contains("Copy theme") && footer_contains(s, "Copied JSON: 3024 Day"))?;
    assert_eq!(fs::read(path.join("copied.txt")).unwrap(), payload);
    assert!(copied.row_text(0).contains("Compare"));
    terminal.send(Key::Esc)?;
    support::wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Explore") && !theme_list_visible(s)
    })?;
    quit(&mut terminal)
}

#[rstest]
#[case("", false)]
#[case("z", false)]
#[case("", true)]
#[case("z", true)]
#[case("c", true)]
#[case("zc", true)]
fn empty_results_copy_the_pin_or_report_no_target(
    #[case] keys: &str,
    #[case] pin: bool,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();
    let mut terminal = start(path, 80, 24)?;
    if pin {
        terminal.send(Key::Char('p'))?;
    }
    terminal.send_str(keys)?;
    terminal.send(Key::Char('/'))?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("NO_SUCH_THEME")?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("0 / 348") && !s.cursor_visible())?;
    terminal.send(Key::Char('y'))?;
    if pin {
        terminal.wait_frame(|s| s.contains("Target: Pin") && s.contains("Theme: 3024 Day"))?;
        terminal.send(Key::Char('t'))?;
        terminal.send(Key::Enter)?;
        terminal.wait_frame(|s| footer_contains(s, "Copied Name: 3024 Day"))?;
        assert_eq!(
            fs::read_to_string(path.join("copied.txt")).unwrap(),
            "3024 Day"
        );
    } else {
        terminal.wait_frame(|s| footer_contains(s, "No theme to copy"))?;
        assert!(!path.join("copied.txt").exists());
    }
    quit(&mut terminal)
}

#[test]
fn same_theme_has_explicit_roles_and_long_names_are_copied_in_full() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();
    let mut terminal = start(path, 80, 24)?;
    terminal.send(Key::Char('/'))?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("LiquidCarbonTransparentInverse")?;
    terminal.send(Key::Enter)?;
    terminal.send_str("py")?;
    let selected = terminal.wait_frame(|s| {
        s.contains("Target: Current") && s.contains("Theme: LiquidCarbonTransparentInverse")
    })?;
    insta::assert_snapshot!("copy_selection_minimum", selected.with_styles());
    terminal.send(Key::Char('t'))?;
    terminal.wait_frame(|s| {
        s.contains("Target: Pin") && s.contains("Theme: LiquidCarbonTransparentInverse")
    })?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| {
        s.row_text(23)
            .contains("Copied Name: LiquidCarbonTransparentInverse")
    })?;
    assert_eq!(
        fs::read_to_string(path.join("copied.txt")).unwrap(),
        "LiquidCarbonTransparentInverse"
    );
    quit(&mut terminal)
}
