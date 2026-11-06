mod support;

use std::{fs, path::Path};

use rstest::rstest;
use support::{
    quit, resize_and_wait,
    screen::{
        footer_contains, is_active_label, matching_themes, preview_name_matches, theme_list_visible,
    },
    wait_for,
};
use termlens::{Key, Screen, Terminal};

fn start(path: &Path) -> termlens::Result<(Terminal, Screen)> {
    let mut terminal = support::spawn(80, 24, &path.with_extension("marks.toml"), path)?;
    let screen = view(&mut terminal, "Explore", true, 1, 348)?;
    Ok((terminal, screen))
}

fn view(
    terminal: &mut Terminal,
    mode: &str,
    list_visible: bool,
    position: usize,
    count: usize,
) -> termlens::Result<Screen> {
    let catalog: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../assets/vhs/themes.json")).unwrap();
    let mut names: Vec<&str> = catalog
        .iter()
        .map(|theme| theme["name"].as_str().unwrap())
        .filter(|name| count != 8 || name.to_lowercase().contains("catpp"))
        .collect();
    names.sort_unstable();
    let name = position
        .checked_sub(1)
        .and_then(|index| names.get(index))
        .copied()
        .unwrap_or("No theme selected");
    wait_for(terminal, |s| {
        s.row_text(0).contains(mode)
            && (if mode == "Compare" {
                if count == 0 {
                    s.contains("Current") && s.contains(name)
                } else {
                    preview_name_matches(s, "Current:", name)
                }
            } else {
                s.row_text(4).contains(name)
            })
            && matching_themes(s, count)
            && !s.cursor_visible()
            && !s.contains("· Draft")
            && !s.contains("Help ·")
            && !s.contains("Select preview")
            && !s.contains("Save filter")
            && !s.contains("Selected filter")
            && theme_list_visible(s) == list_visible
            && if list_visible {
                is_active_label(s, "Themes") && !is_active_label(s, "Current")
            } else if mode == "Compare" {
                is_active_label(s, "Current") && !is_active_label(s, "Pin")
            } else {
                count == 0 || is_active_label(s, name)
            }
    })
}

#[rstest]
#[case(false)]
#[case(true)]
fn comparison_keeps_both_list_settings_and_pin_replacement_keeps_the_view(
    #[case] explore_list_open: bool,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let (mut terminal, _) = start(&path)?;
    terminal.send_str("pj")?;
    view(&mut terminal, "Explore", true, 2, 348)?;
    if !explore_list_open {
        terminal.send(Key::Char('z'))?;
        view(&mut terminal, "Explore", false, 2, 348)?;
    }
    terminal.send(Key::Char('c'))?;
    let compared = view(&mut terminal, "Compare", false, 2, 348)?;
    assert!(compared.row_text(4).contains("Pin: 3024 Day"));
    assert!(compared.row_text(4).contains("Current: 3024 Night"));

    terminal.send(Key::F(1))?;
    wait_for(&mut terminal, |s| s.contains("Help · Compare"))?;
    terminal.send(Key::Ctrl('f'))?;
    let help = wait_for(&mut terminal, |s| s.contains("Back to Explore"))?;
    let help_text = (0..24).map(|row| help.row_text(row)).collect::<String>();
    assert!(help_text.contains("Esc  c"));
    assert_eq!(help_text.matches("Back to Explore").count(), 1);
    terminal.send(Key::Esc)?;
    view(&mut terminal, "Compare", false, 2, 348)?;

    terminal.send(Key::Char('z'))?;
    view(&mut terminal, "Compare", true, 2, 348)?;
    assert!(
        !path.exists(),
        "list and view changes must not write preferences"
    );
    terminal.send(Key::Char('v'))?;
    let stacked = wait_for(&mut terminal, |s| {
        s.row_text(13).contains("Current: 3024 Night")
    })?;
    assert!(footer_contains(&stacked, "Esc Explore"));
    terminal.send(Key::Char('c'))?;
    view(&mut terminal, "Explore", explore_list_open, 2, 348)?;
    terminal.send(Key::Char('c'))?;
    view(&mut terminal, "Compare", true, 2, 348)?;
    terminal.send(Key::Esc)?;
    view(&mut terminal, "Explore", explore_list_open, 2, 348)?;

    terminal.send(Key::Char('c'))?;
    view(&mut terminal, "Compare", true, 2, 348)?;
    terminal.send(Key::Char('p'))?;
    wait_for(&mut terminal, |s| s.contains("Pin: 3024 Night"))?;
    terminal.send(Key::Char('p'))?;
    let unpinned = view(&mut terminal, "Explore", explore_list_open, 2, 348)?;
    assert!(!unpinned.row_text(1).contains("Pin  "));
    terminal.send(Key::Char('c'))?;
    wait_for(&mut terminal, |s| s.contains("Pin a theme first"))?;
    quit(&mut terminal)?;

    let (mut terminal, initial) = start(&path)?;
    assert!(!initial.row_text(1).contains("Pin  "));
    terminal.send_str("pjc")?;
    view(&mut terminal, "Compare", false, 2, 348)?;
    quit(&mut terminal)
}

#[rstest]
#[case(false)]
#[case(true)]
fn hidden_navigation_preserves_the_viewport_and_uses_the_current_size(
    #[case] compare: bool,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let (mut terminal, _) = start(&path)?;
    terminal.send_str(&"j".repeat(49))?;
    terminal.send(Key::Char('p'))?;
    let before = wait_for(&mut terminal, |s| {
        theme_list_visible(s)
            && s.row_text(4).contains("Catppuccin Mocha")
            && preview_name_matches(s, "Pin", "Catppuccin Mocha")
    })?;
    let mode = if compare { "Compare" } else { "Explore" };
    let hide_key = if compare { 'c' } else { 'z' };
    terminal.send(Key::Char(hide_key))?;
    let hidden = view(&mut terminal, mode, false, 50, 348)?;
    if !compare {
        insta::assert_snapshot!("explore_list_hidden", hidden.with_styles());
    }
    if compare {
        terminal.send(Key::Esc)?;
    } else {
        terminal.send(Key::Char('z'))?;
    }
    let restored = view(&mut terminal, "Explore", true, 50, 348)?;
    assert_eq!(
        before.with_styles().to_string(),
        restored.with_styles().to_string()
    );
    assert!(!path.exists());

    terminal.send(Key::Char(hide_key))?;
    view(&mut terminal, mode, false, 50, 348)?;
    terminal.send(Key::Ctrl('f'))?;
    view(&mut terminal, mode, false, 66, 348)?;
    resize_and_wait(&mut terminal, 120, 36, |s| footer_contains(s, "Quit"))?;
    terminal.send(Key::Ctrl('f'))?;
    view(&mut terminal, mode, false, 94, 348)?;
    terminal.send(Key::Char('G'))?;
    view(&mut terminal, mode, false, 348, 348)?;
    terminal.send(Key::Ctrl('b'))?;
    view(&mut terminal, mode, false, 320, 348)?;
    resize_and_wait(&mut terminal, 59, 22, |s| s.contains("Current:  59x22"))?;
    let minimum = resize_and_wait(&mut terminal, 60, 22, |s| footer_contains(s, "Quit"))?;
    assert!(!theme_list_visible(&minimum));
    if !compare {
        assert!(footer_contains(&minimum, "j/k Move"));
        assert!(footer_contains(&minimum, "q Quit"));
        assert!(minimum.row_text(21).trim_end().ends_with("F1 Help"));
        assert!(!footer_contains(&minimum, "z List"));
        terminal.send(Key::Esc)?;
        let unchanged = view(&mut terminal, "Explore", false, 320, 348)?;
        assert_eq!(
            unchanged.with_styles().to_string(),
            minimum.with_styles().to_string()
        );
        terminal.send(Key::F(1))?;
        wait_for(&mut terminal, |s| s.contains("Help · Explore"))?;
        terminal.send(Key::Ctrl('f'))?;
        let help = wait_for(&mut terminal, |s| s.contains("Show or hide the theme list"))?;
        assert!(help.contains("Show or hide the theme list"));
        assert!(!help.contains("Back to Explore"));
        terminal.send(Key::Esc)?;
        view(&mut terminal, "Explore", false, 320, 348)?;
    }
    let resized = resize_and_wait(&mut terminal, 80, 24, |s| footer_contains(s, "Quit"))?;
    assert!(!theme_list_visible(&resized));
    if compare {
        terminal.send(Key::Esc)?;
    } else {
        terminal.send(Key::Char('z'))?;
    }
    let restored = view(&mut terminal, "Explore", true, 320, 348)?;
    assert!((6..22).any(|y| restored.cell(y, 6).unwrap().style().reverse));
    terminal.send(Key::Char('g'))?;
    view(&mut terminal, "Explore", true, 1, 348)?;
    quit(&mut terminal)
}

#[rstest]
#[case("Explore", true, "pj")]
#[case("Explore", false, "pjz")]
#[case("Compare", false, "pjc")]
#[case("Compare", true, "pjcz")]
fn editors_and_menus_return_to_the_original_view(
    #[case] mode: &str,
    #[case] list_visible: bool,
    #[case] keys: &str,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let (mut terminal, _) = start(&path)?;
    terminal.send_str(keys)?;
    let before = view(&mut terminal, mode, list_visible, 2, 348)?;
    for (key, marker) in [
        (Key::Char('f'), "· Draft"),
        (Key::Char('s'), "Saved filters"),
        (Key::Char('S'), "Save filter"),
        (Key::Char('P'), "Select preview"),
        (Key::F(1), "Help ·"),
    ] {
        terminal.send(key)?;
        terminal.wait_frame(|s| s.contains(marker))?;
        terminal.send(Key::Esc)?;
        let restored = view(&mut terminal, mode, list_visible, 2, 348)?;
        assert_eq!(
            before.with_styles().to_string(),
            restored.with_styles().to_string()
        );
    }
    terminal.send(Key::Char('s'))?;
    terminal.wait_frame(|s| s.contains("Selected filter"))?;
    terminal.send(Key::Enter)?;
    view(&mut terminal, mode, list_visible, 2, 348)?;
    assert!(!path.exists());
    terminal.send(Key::Char('/'))?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("catpp")?;
    terminal.wait_frame(|s| s.cursor_visible() && s.contains("8 / 348") && s.contains("Themes"))?;
    terminal.send(Key::Esc)?;
    let restored = view(&mut terminal, mode, list_visible, 2, 348)?;
    assert_eq!(
        before.with_styles().to_string(),
        restored.with_styles().to_string()
    );
    terminal.send(Key::Char('/'))?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("catpp")?;
    terminal.wait_frame(|s| s.cursor_visible() && s.contains("8 / 348"))?;
    terminal.send(Key::Enter)?;
    view(&mut terminal, mode, list_visible, 1, 8)?;
    let value: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(
        value["session"]["query"].as_str(),
        Some("catpp ignore:false")
    );
    terminal.send(Key::Char('S'))?;
    terminal.wait_frame(|s| s.contains("Save filter") && s.cursor_visible())?;
    terminal.send_str("Selected")?;
    terminal.send(Key::Enter)?;
    let saved = view(&mut terminal, mode, list_visible, 1, 8)?;
    assert!(saved.contains("Filter saved"));
    terminal.send(Key::Char('s'))?;
    terminal.wait_frame(|s| s.contains("Saved filters") && s.contains("Initial [built-in]"))?;
    terminal.send(Key::Enter)?;
    view(&mut terminal, mode, list_visible, 47, 348)?;
    if mode == "Compare" {
        terminal.send(Key::Esc)?;
        view(&mut terminal, "Explore", true, 47, 348)?;
    }
    quit(&mut terminal)
}
