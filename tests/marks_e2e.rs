mod support;

use std::{collections::BTreeMap, fs, path::Path, process::Command};

use rstest::rstest;
use serde::Deserialize;
use support::{
    apply_query as search, quit_with,
    screen::{
        comparing, comparison_layout, footer_contains, matching_themes, preview_name_matches,
        shows_current_theme,
    },
};
use termlens::{Key, Screen, Terminal};

fn start(path: &Path) -> termlens::Result<(Terminal, Screen)> {
    support::start(80, 24, path, &path.with_extension("workspace.toml"))
}

fn saved(path: &Path) -> BTreeMap<String, String> {
    #[derive(Deserialize)]
    struct Saved {
        version: u32,
        themes: BTreeMap<String, String>,
    }
    let saved: Saved = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(saved.version, 1);
    saved.themes
}

#[test]
fn marks_survive_restart_keep_unknown_names_and_combine_with_filters() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("marks.toml");
    fs::write(
        &path,
        r#"
version = 1
[themes]
"Removed Theme" = "favorite"
"#,
    )
    .unwrap();
    let (mut first, _) = start(&path)?;
    first.send(Key::Char('m'))?;
    first.wait_frame(|s| s.contains("+ 3024 Day") && s.contains("Favorite saved"))?;
    first.send_str("pjx")?;
    first.wait_frame(|s| s.contains("Ignore saved") && s.contains("347 / 348"))?;
    first.send_str("cv]")?;
    first.wait_frame(|s| comparing(s) && s.contains("Shell"))?;
    quit_with(&mut first, Key::Char('q'))?;
    assert_eq!(
        saved(&path),
        BTreeMap::from([
            ("Removed Theme".into(), "favorite".into()),
            ("3024 Day".into(), "favorite".into()),
            ("3024 Night".into(), "ignored".into()),
        ])
    );
    let (mut second, screen) = start(&path)?;
    assert!(screen.contains("347 / 348") && screen.contains("Shell"));
    assert!(screen.contains("+ 3024 Day"));
    assert!(!screen.contains("*+ 3024 Day"));
    assert!(!comparing(&screen));
    search(&mut second, "favorite:true", 1)?;
    let filtered = search(&mut second, "favorite:false ignore:true type:dark", 1)?;
    assert!(filtered.contains("- 3024 Night"));
    second.send(Key::Char('x'))?;
    second.wait_frame(|s| s.contains("Mark cleared") && s.contains("0 / 348"))?;
    second.send(Key::Char('R'))?;
    second.wait_frame(|s| matching_themes(s, 348))?;
    quit_with(&mut second, Key::Char('q'))?;
    assert!(!saved(&path).contains_key("3024 Night"));
    assert_eq!(saved(&path)["Removed Theme"], "favorite");
    Ok(())
}

#[rstest]
#[case(false)]
#[case(true)]
fn exclusive_marks_keep_filtered_pins_and_recover_through_filters(
    #[case] vertical: bool,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("marks.toml");
    let (mut terminal, _) = start(&path)?;
    search(&mut terminal, "catpp", 8)?;
    terminal.send_str("mpc")?;
    if vertical {
        terminal.send(Key::Char('v'))?;
    }
    terminal.wait_frame(|s| {
        comparing(s)
            && preview_name_matches(s, "Pin:", "Catppuccin Frappe")
            && comparison_layout(s, vertical)
    })?;
    terminal.send(Key::Char('x'))?;
    let removed = terminal.wait_frame(|s| s.contains("7 / 348") && s.contains("Ignore saved"))?;
    assert!(removed.contains("7 / 348"));
    assert!(removed.row_text(23).contains("Ignore saved"));
    assert!(!removed.contains("*  Catppuccin Frappe"));
    assert_eq!(saved(&path)["Catppuccin Frappe"], "ignored");
    terminal.send(Key::Char('f'))?;
    terminal.send(Key::Tab)?;
    terminal.send(Key::Char('l'))?;
    terminal.wait_frame(|s| s.contains("Favorites") && s.contains("No matching themes"))?;
    terminal.send(Key::Char('h'))?;
    terminal.send(Key::Tab)?;
    terminal.send(Key::Char('l'))?;
    terminal.wait_frame(|s| {
        s.contains("Ignored included")
            && s.contains("*- Catppuccin Frappe")
            && s.row_text(1).contains("8 / 348")
    })?;
    terminal.send(Key::Char('l'))?;
    let only = terminal
        .wait_frame(|s| s.contains("Ignored only") && s.contains("*- Catppuccin Frappe"))?;
    assert!(only.contains("Only"));
    assert!(only.contains("Matches"));
    if !vertical {
        insta::assert_snapshot!("filters_ignored_minimum", only.with_styles());
    }
    terminal.send(Key::Enter)?;
    terminal.send(Key::Char('x'))?;
    terminal.wait_frame(|s| {
        s.contains("Mark cleared") && s.contains("Current") && s.contains("No theme selected")
    })?;
    terminal.send(Key::Char('R'))?;
    terminal.wait_frame(|s| {
        matching_themes(s, 348)
            && comparing(s)
            && preview_name_matches(s, "Pin:", "Catppuccin Frappe")
    })?;
    let filtered = search(&mut terminal, "favorite:true ignore:true", 0)?;
    assert!(!filtered.contains("Unknown filter"));
    terminal.send(Key::Char('R'))?;
    terminal.wait_frame(|s| matching_themes(s, 348))?;
    quit_with(&mut terminal, Key::Char('q'))
}

#[test]
fn failed_mark_save_keeps_the_mark_unchanged_and_can_be_repeated() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("marks.toml");
    let (mut terminal, _) = start(&path)?;
    fs::create_dir(&path).unwrap();
    terminal.send(Key::Char('m'))?;
    let failed = terminal.wait_frame(|s| footer_contains(s, "Could not save marks"))?;
    assert!(!failed.contains("+ 3024 Day"));
    fs::remove_dir(&path).unwrap();
    terminal.send(Key::Char('m'))?;
    terminal.wait_frame(|s| footer_contains(s, "3024 Day: Favorite saved"))?;
    assert_eq!(
        saved(&path),
        BTreeMap::from([("3024 Day".into(), "favorite".into())])
    );
    quit_with(&mut terminal, Key::Char('q'))
}

#[test]
fn removing_favorites_at_the_end_selects_the_previous_theme_then_zero() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("marks.toml");
    fs::write(
        &path,
        r#"
version = 1
[themes]
"3024 Day" = "favorite"
"3024 Night" = "favorite"
"#,
    )
    .unwrap();
    let (mut terminal, _) = start(&path)?;
    search(&mut terminal, "favorite:true", 2)?;
    terminal.send(Key::Char('G'))?;
    terminal.wait_frame(|s| shows_current_theme(s, "3024 Night"))?;
    terminal.send(Key::Char('m'))?;
    terminal.wait_frame(|s| s.contains("1 / 348") && s.row_text(4).contains("3024 Day"))?;
    terminal.send(Key::Char('m'))?;
    terminal.wait_frame(|s| s.contains("0 / 348") && s.contains("No theme selected"))?;
    assert_eq!(saved(&path), BTreeMap::new());
    quit_with(&mut terminal, Key::Char('q'))
}

#[test]
fn an_empty_override_fails_before_entering_terminal_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_asinus"))
        .env_clear()
        .env("ASINUS_STATE_PATH", "")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("empty state path"));
}

#[rstest]
#[case(false)]
#[case(true)]
fn default_storage_respects_data_home_and_is_created_on_first_mark(
    #[case] custom_data_home: bool,
) -> termlens::Result<()> {
    let home = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let xdg = custom_data_home.then_some(data.path());
    let path = xdg
        .map_or_else(|| home.path().join(".local/share"), Path::to_path_buf)
        .join("asinus/marks.toml");
    let mut terminal = termlens::bin!(
        "asinus",
        env_clear(),
        env("HOME", home.path()),
        envs(xdg.into_iter().map(|path| ("XDG_DATA_HOME", path)))
    )?;
    terminal.wait_frame(|s| matching_themes(s, 348))?;
    assert!(!path.exists());
    terminal.send(Key::Char('m'))?;
    terminal.wait_frame(|s| s.contains("Favorite saved"))?;
    assert_eq!(saved(&path)["3024 Day"], "favorite");
    quit_with(&mut terminal, Key::Char('q'))
}
