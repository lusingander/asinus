mod support;

use std::{fs, path::Path};

use rstest::rstest;
use support::{
    apply_query as query, quit, resize_and_wait,
    screen::{
        footer_contains, is_active_label, is_selected, matching_themes, shows_current_theme,
        theme_list_visible,
    },
};
use termlens::{Color, Key, Screen, Terminal};

fn start(path: &Path) -> termlens::Result<(Terminal, Screen)> {
    start_at(path, 80, 24)
}

fn start_at(path: &Path, width: u16, height: u16) -> termlens::Result<(Terminal, Screen)> {
    support::start(width, height, &path.with_extension("marks.toml"), path)
}

fn disk(path: &Path) -> toml::Value {
    toml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn save(terminal: &mut Terminal, name: &str) -> termlens::Result<Screen> {
    terminal.send(Key::Char('S'))?;
    terminal.wait_frame(|s| s.contains("Save filter") && s.cursor_visible())?;
    terminal.send_str(name)?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| !s.cursor_visible() && footer_contains(s, "Filter saved"))
}

fn manager(terminal: &mut Terminal) -> termlens::Result<Screen> {
    terminal.send_str("s")?;
    terminal.wait_frame(|s| is_active_label(s, "Saved filters") && s.contains("Selected filter"))
}

fn close_manager(terminal: &mut Terminal) -> termlens::Result<Screen> {
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !s.contains("Saved filters") && !s.contains("Selected filter"))
}

#[test]
fn saved_filter_management_preserves_the_last_confirmed_condition() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let (mut terminal, _) = start(&path)?;
    query(&mut terminal, "type:dark", 298)?;
    let saved = save(&mut terminal, "Dark")?;
    assert!(saved.row_text(4).contains("Themes"));
    query(&mut terminal, "type:light", 50)?;
    save(&mut terminal, "Light")?;
    let list = manager(&mut terminal)?;
    assert_eq!(list.cell(0, 1).unwrap().style().fg, Color::Default);
    assert!(!is_active_label(&list, "Selected filter"));
    terminal.send_str("je")?;
    let rename = terminal.wait_frame(|s| s.contains("Rename filter") && s.cursor_visible())?;
    assert!(!is_active_label(&rename, "Saved filters"));
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("Night")?;
    terminal.send(Key::Enter)?;
    let renamed =
        terminal.wait_frame(|s| s.contains("Filter renamed") && is_selected(s, "Night"))?;
    assert!(is_active_label(&renamed, "Saved filters"));
    assert_eq!(
        disk(&path)["saved_filters"][0]["name"].as_str(),
        Some("Night")
    );
    terminal.send(Key::Char('D'))?;
    let delete =
        terminal.wait_frame(|s| s.contains("Delete filter?") && s.contains("Name: Night"))?;
    assert!(!is_active_label(&delete, "Saved filters"));
    insta::assert_snapshot!("delete_filter_minimum", delete.with_styles());
    terminal.send(Key::Esc)?;
    let cancelled =
        terminal.wait_frame(|s| !s.contains("Delete filter?") && is_selected(s, "Night"))?;
    assert!(is_active_label(&cancelled, "Saved filters"));
    assert_eq!(disk(&path)["saved_filters"].as_array().unwrap().len(), 2);
    terminal.send(Key::Char('D'))?;
    terminal.wait_frame(|s| s.contains("Delete filter?"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("Filter deleted") && !is_selected(s, "Night"))?;
    assert_eq!(disk(&path)["saved_filters"].as_array().unwrap().len(), 1);
    save(&mut terminal, "Night")?;
    assert_eq!(
        disk(&path)["saved_filters"][1]["name"].as_str(),
        Some("Night")
    );
    assert_eq!(
        disk(&path)["session"]["query"].as_str(),
        Some("type:light ignore:false")
    );
    let closed = close_manager(&mut terminal)?;
    assert!(closed.contains("Themes") && closed.contains("50 / 348"));
    quit(&mut terminal)?;

    let (mut terminal, initial) = start(&path)?;
    assert!(initial.contains("50 / 348"));
    manager(&mut terminal)?;
    terminal.send_str("j")?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("50 / 348") && s.contains("Themes"))?;
    quit(&mut terminal)
}

#[test]
fn initial_is_always_available_and_stays_distinct_from_a_saved_filter_with_the_same_name()
-> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let (mut terminal, _) = start(&path)?;
    query(&mut terminal, "catpp type:dark", 6)?;
    terminal.send_str("pcv]x")?;
    terminal.wait_frame(|s| s.contains("5 / 348") && s.contains("Ignore saved"))?;
    let initial = manager(&mut terminal)?;
    assert!(initial.row_text(4).contains("1 / 1") && initial.contains("347 matching themes"));
    insta::assert_snapshot!("initial_only_minimum", initial.with_styles());
    let footer = initial.row_text(23);
    assert!(footer.contains("Enter Apply"));
    assert!(!footer.contains("Rename") && !footer.contains("Delete"));
    let unchanged = fs::read_to_string(&path).unwrap();
    terminal.send_str("eD")?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| {
        s.row_text(4).contains("Catppuccin Frappe")
            && s.row_text(13).contains("Catppuccin Macchiato")
            && s.contains("347 / 348")
            && s.contains("Shell")
    })?;
    let value = disk(&path);
    assert_eq!(value["session"]["query"].as_str(), Some("ignore:false"));
    assert!(value["saved_filters"].as_array().unwrap().is_empty());
    assert_ne!(fs::read_to_string(&path).unwrap(), unchanged);
    manager(&mut terminal)?;
    save(&mut terminal, "Initial")?;
    terminal.send(Key::Char('e'))?;
    terminal.wait_frame(|s| s.contains("Rename filter") && s.cursor_visible())?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !s.cursor_visible())?;
    terminal.send(Key::Char('g'))?;
    let initial = terminal.wait_frame(|s| s.row_text(4).contains("1 / 2"))?;
    assert!(!initial.row_text(23).contains("Rename"));
    assert!(!initial.row_text(23).contains("Delete"));
    let unchanged = fs::read_to_string(&path).unwrap();
    terminal.send_str("eDj")?;
    terminal.wait_frame(|s| footer_contains(s, "e Rename") && s.row_text(4).contains("2 / 2"))?;
    assert_eq!(fs::read_to_string(&path).unwrap(), unchanged);
    terminal.send(Key::Char('D'))?;
    terminal.wait_frame(|s| s.contains("Delete filter?"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("Filter deleted") && s.contains("Initial [built-in]"))?;
    assert!(disk(&path)["saved_filters"].as_array().unwrap().is_empty());
    terminal.send(Key::Enter)?;
    terminal
        .wait_frame(|s| s.contains("347 / 348") && s.row_text(4).contains("Catppuccin Frappe"))?;
    quit(&mut terminal)?;
    let (mut terminal, initial) = start(&path)?;
    assert!(initial.contains("347 / 348") && initial.contains("Shell"));
    terminal.send(Key::Char('c'))?;
    terminal.wait_frame(|s| s.contains("Pin a theme first"))?;
    quit(&mut terminal)
}

#[test]
fn dialogs_validate_names_preserve_the_origin_and_keep_comparison_when_applying()
-> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let (mut terminal, _) = start(&path)?;
    query(&mut terminal, "catpp", 8)?;
    terminal.send_str("pcv")?;
    terminal.wait_frame(|s| s.row_text(13).contains("Catppuccin Frappe"))?;
    terminal.send(Key::Char('S'))?;
    let initial = terminal.wait_frame(|s| s.contains("Save filter") && s.cursor_visible())?;
    insta::assert_snapshot!("save_from_comparison_minimum", initial.with_styles());
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("Enter a name"))?;
    terminal.send_str("q S n e d")?;
    terminal.wait_frame(|s| s.contains("q S n e d") && !s.contains("Enter a name"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !s.cursor_visible() && s.row_text(13).contains("Catppuccin Frappe"))?;
    save(&mut terminal, "Cats")?;
    terminal.send(Key::Char('S'))?;
    terminal.wait_frame(|s| s.cursor_visible())?;
    terminal.send_str("Cats")?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("Name already exists"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !s.contains("Save filter") && !s.cursor_visible())?;
    query(&mut terminal, "catpp mocha", 2)?;
    manager(&mut terminal)?;
    terminal.send(Key::Char('j'))?;
    terminal.send(Key::Char('u'))?;
    terminal.send(Key::Enter)?;
    terminal
        .wait_frame(|s| s.row_text(4).contains("Catppuccin Frappe") && s.contains("8 / 348"))?;
    assert_eq!(
        disk(&path)["saved_filters"][0]["query"].as_str(),
        Some("catpp ignore:false")
    );
    query(&mut terminal, "catpp mocha", 2)?;
    manager(&mut terminal)?;
    terminal.send(Key::Char('j'))?;
    let picked =
        terminal.wait_frame(|s| s.row_text(4).contains("2 / 2") && is_selected(s, "Cats"))?;
    assert!(!picked.row_text(23).contains("Update"));
    terminal.send(Key::Char('D'))?;
    terminal.wait_frame(|s| s.contains("Delete filter?"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("Filter deleted"))?;
    save(&mut terminal, "Cats")?;
    assert_eq!(
        disk(&path)["saved_filters"][0]["query"].as_str(),
        Some("catpp mocha ignore:false")
    );
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| {
        s.contains("2 / 348")
            && s.row_text(4).contains("Catppuccin Frappe")
            && s.row_text(13).contains("Catppuccin Mocha")
    })?;
    quit(&mut terminal)
}

#[test]
fn failed_save_keeps_the_dialog_and_current_condition_for_another_attempt() -> termlens::Result<()>
{
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let (mut terminal, _) = start(&path)?;
    fs::create_dir(&path).unwrap();
    let failed = query(&mut terminal, "type:dark", 298)?;
    assert!(footer_contains(&failed, "Could not save workspace"));
    terminal.send(Key::Char('S'))?;
    terminal.wait_frame(|s| s.contains("Save filter") && s.cursor_visible())?;
    terminal.send_str("Retried")?;
    terminal.send(Key::Enter)?;
    let failed = terminal
        .wait_frame(|s| footer_contains(s, "Could not save workspace") && s.cursor_visible())?;
    assert!(failed.contains("Retried"));
    fs::remove_dir(&path).unwrap();
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| footer_contains(s, "Filter saved") && !s.cursor_visible())?;
    let value = disk(&path);
    assert_eq!(value["saved_filters"][0]["name"].as_str(), Some("Retried"));
    assert_eq!(
        value["session"]["query"].as_str(),
        Some("type:dark ignore:false")
    );
    quit(&mut terminal)
}

#[rstest]
#[case(false)]
#[case(true)]
fn a_single_store_failure_makes_the_entire_session_read_only(
    #[case] lock_conflict: bool,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let marks = dir.path().join("marks.toml");
    let workspace = dir.path().join("workspace.toml");
    let mut writer = if lock_conflict {
        // Only the workspace path is shared between the two processes.
        let (writer, _) =
            support::start(80, 24, &dir.path().join("writer-marks.toml"), &workspace)?;
        Some(writer)
    } else {
        fs::write(&workspace, "broken = [").unwrap();
        None
    };
    if let Some(writer) = &mut writer {
        save(writer, "Initial view")?;
    }

    let (mut reader, initial) = support::start(80, 24, &marks, &workspace)?;
    assert!(footer_contains(&initial, "Read-only"));
    if let Some(writer) = &mut writer {
        let opened = manager(&mut reader)?;
        assert!(opened.contains("Initial view"));
        close_manager(&mut reader)?;
        quit(writer)?;
    }
    let before = fs::read(&workspace).unwrap();
    reader.send_str("mS")?;
    let filtered = query(&mut reader, "type:dark", 298)?;
    assert!(!filtered.contains("Read-only"));
    assert!(!filtered.contains("+ 3024 Day"));
    quit(&mut reader)?;
    assert!(!marks.exists());
    assert_eq!(fs::read(&workspace).unwrap(), before);
    Ok(())
}

#[test]
fn initial_only_and_paged_managers_allow_unicode_names_and_zero_match_filters()
-> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let (mut terminal, _) = start(&path)?;
    let opened = manager(&mut terminal)?;
    assert!(opened.contains("Initial [built-in]") && footer_contains(&opened, "S Save"));
    let saved = save(&mut terminal, "From manager")?;
    assert!(saved.contains("Selected filter"));
    assert_eq!(
        disk(&path)["saved_filters"][0]["query"].as_str(),
        Some("ignore:false")
    );
    let closed = close_manager(&mut terminal)?;
    assert!(closed.contains("Themes"));
    quit(&mut terminal)?;
    let long_name = "夜".repeat(40);
    let mut value = toml::toml! {
        version = 1
        saved_filters = []
        [session]
        preset = "palette"
        layout = "horizontal"
    };
    for index in 0..24 {
        value["saved_filters"]
            .as_array_mut()
            .unwrap()
            .push(toml::Value::Table(toml::Table::from_iter([
                (
                    "name".into(),
                    if index == 23 {
                        long_name.clone()
                    } else {
                        format!("Filter {index:02}")
                    }
                    .into(),
                ),
                (
                    "query".into(),
                    if index == 23 {
                        "missing-theme-000"
                    } else {
                        "type:dark"
                    }
                    .into(),
                ),
            ])));
    }
    fs::write(&path, toml::to_string(&value).unwrap()).unwrap();
    let (mut terminal, _) = start(&path)?;
    terminal.send(Key::Char('s'))?;
    terminal.wait_frame(|s| s.contains("Saved filters") && s.contains("1 / 25"))?;
    terminal.send(Key::Ctrl('f'))?;
    terminal.wait_frame(|s| {
        s.contains("Saved filters") && s.contains("17 / 25") && s.contains("Filter 15")
    })?;
    terminal.send(Key::Ctrl('b'))?;
    terminal.wait_frame(|s| s.contains("1 / 25"))?;
    terminal.send(Key::Char('G'))?;
    let last = terminal.wait_frame(|s| {
        s.contains("Saved filters") && s.contains("25 / 25") && s.contains("0 matching themes")
    })?;
    assert!(last.contains("Selected filter"));
    terminal.send(Key::Char('g'))?;
    terminal.wait_frame(|s| s.row_text(4).contains("1 / 25"))?;
    terminal.send(Key::Char('G'))?;
    let last = terminal.wait_frame(|s| s.contains("0 matching themes") && s.contains("25 / 25"))?;
    assert!(last.row_text(4).contains("Saved filters"));
    terminal.send(Key::Char('e'))?;
    terminal.wait_frame(|s| s.contains("Rename filter") && s.cursor_visible())?;
    terminal.send(Key::Char('夜'))?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("Use 1–40 characters"))?;
    terminal.send(Key::Backspace)?;
    terminal.wait_frame(|s| !s.contains("Use 1–40 characters"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !s.contains("Rename filter") && !s.cursor_visible())?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("No matching themes") && s.contains("0 / 348"))?;
    let saved = save(&mut terminal, "No results")?;
    assert!(saved.contains("No matching themes"));
    quit(&mut terminal)?;
    let (mut terminal, initial) = start(&path)?;
    assert!(initial.contains("No matching themes") && initial.contains("0 / 348"));
    terminal.send(Key::Char('R'))?;
    terminal.wait_frame(|s| matching_themes(s, 348))?;
    quit(&mut terminal)
}

#[rstest]
#[case(false)]
#[case(true)]
fn confirmed_conditions_and_preferences_survive_restart_but_drafts_and_pins_do_not(
    #[case] filters: bool,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let (mut first, _) = start(&path)?;
    assert!(!path.exists());
    query(&mut first, "catpp type:dark", 6)?;
    first.send_str("]pcv")?;
    first.wait_frame(|s| s.row_text(13).contains("Catppuccin Frappe") && s.contains("Shell"))?;
    if filters {
        first.send_str("fl")?;
        first.wait_frame(|s| s.contains("2 / 348") && s.contains("· Draft"))?;
        first.send(Key::BackTab)?;
        first.send(Key::Char('"'))?;
    } else {
        first.send(Key::Char('/'))?;
        first.send(Key::Ctrl('u'))?;
        first.send_str("type:")?;
    }
    first.wait_frame(|s| s.cursor_visible() && s.contains("Last valid results"))?;
    quit(&mut first)?;
    assert_eq!(
        disk(&path)["session"]["query"].as_str(),
        Some("catpp type:dark ignore:false")
    );
    let (mut second, initial) = start(&path)?;
    assert!(initial.contains("6 / 348") && initial.contains("Shell"));
    assert!(!initial.contains("*Catpp"));
    second.send(Key::Char('c'))?;
    second.wait_frame(|s| s.contains("Pin a theme first"))?;
    second.send_str("pc")?;
    second.wait_frame(|s| s.row_text(13).contains("Catppuccin Frappe"))?;
    quit(&mut second)
}

#[rstest]
#[case(false)]
#[case(true)]
fn cancelled_edits_and_reset_persist_only_the_confirmed_condition(
    #[case] filters: bool,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let (mut terminal, _) = start(&path)?;
    query(&mut terminal, "type:dark", 298)?;
    if filters {
        terminal.send_str("fl")?;
    } else {
        terminal.send(Key::Char('/'))?;
        terminal.send(Key::Ctrl('u'))?;
        terminal.send_str("type:light")?;
    }
    terminal.wait_frame(|s| s.contains("50 / 348"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !s.cursor_visible() && s.contains("298 / 348"))?;
    assert_eq!(
        disk(&path)["session"]["query"].as_str(),
        Some("type:dark ignore:false")
    );
    terminal.send(Key::Char('R'))?;
    terminal.wait_frame(|s| matching_themes(s, 348))?;
    quit(&mut terminal)?;
    assert_eq!(
        disk(&path)["session"]["query"].as_str(),
        Some("ignore:false")
    );
    Ok(())
}

#[rstest]
#[case("Explore", true, "pj")]
#[case("Explore", false, "pjz")]
#[case("Compare", false, "pjc")]
#[case("Compare", true, "pjcz")]
fn saved_screen_keeps_the_save_source_and_returns_to_the_original_view(
    #[case] mode: &str,
    #[case] list_visible: bool,
    #[case] keys: &str,
    #[values(80, 120)] width: u16,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    fs::write(
        &path,
        r#"version = 1
saved_filters = [{ name = "Dark", query = "type:dark" }, { name = "Initial", query = "type:light" }]
[session]
preset = "palette"
layout = "horizontal"
"#,
    )
    .unwrap();
    let (mut terminal, _) = start_at(&path, width, if width == 80 { 24 } else { 36 })?;
    terminal.send_str(keys)?;
    let before = support::wait_for(&mut terminal, |s| {
        s.row_text(0).contains(mode)
            && theme_list_visible(s) == list_visible
            && shows_current_theme(s, "3024 Night")
    })?;
    let disk_before = fs::read_to_string(&path).unwrap();
    terminal.send(Key::Char('s'))?;
    terminal.wait_frame(|s| {
        s.contains("Saved filters")
            && s.contains("Initial [built-in]")
            && s.contains("348 matching themes")
    })?;
    terminal.send(Key::Char('j'))?;
    let picked = terminal.wait_frame(|s| {
        s.contains("Saved filters")
            && s.contains("type:dark")
            && is_selected(s, "Dark")
            && s.contains("298 matching themes")
    })?;
    assert!(matching_themes(&picked, 348));
    assert!(is_active_label(&picked, "Saved filters"));
    assert_eq!(fs::read_to_string(&path).unwrap(), disk_before);
    if mode == "Explore" && !list_visible && width == 80 {
        resize_and_wait(&mut terminal, 59, 22, |s| s.contains("Terminal too small"))?;
        resize_and_wait(&mut terminal, 60, 22, |s| {
            is_active_label(s, "Saved filters") && s.contains("298 matching themes")
        })?;
        resize_and_wait(&mut terminal, 120, 36, |s| {
            is_active_label(s, "Saved filters")
                && s.contains("298 matching themes")
                && footer_contains(s, "Quit")
        })?;
        resize_and_wait(&mut terminal, 80, 24, |s| {
            is_active_label(s, "Saved filters")
                && s.contains("298 matching themes")
                && footer_contains(s, "Quit")
        })?;
    }
    for key in "mdpczR[]".chars() {
        terminal.send(Key::Char(key))?;
    }
    terminal.send(Key::F(1))?;
    let help = terminal
        .wait_frame(|s| s.contains("Help · Saved filters") && s.contains("Saved conditions"))?;
    assert!(!help.contains("Manage"));
    assert!(!is_active_label(&help, "Saved filters"));
    terminal.send(Key::Esc)?;
    let picked = terminal.wait_frame(|s| !s.contains("Help ·") && s.contains("Saved filters"))?;
    assert!(picked.contains("298 matching themes"));
    assert!(is_active_label(&picked, "Saved filters"));
    let (row, col) = picked.find("type:dark").unwrap();
    for offset in 0..5 {
        let style = picked.cell(row, col + offset).unwrap().style();
        assert_eq!(style.fg, Color::Indexed(2));
        assert!(!style.bold);
    }
    assert_eq!(
        picked.cell(row, col + 5).unwrap().style().fg,
        Color::Default
    );
    if mode == "Compare" && !list_visible {
        if width == 80 {
            insta::assert_snapshot!("saved_screen_minimum", picked.with_styles());
        } else {
            insta::assert_snapshot!("saved_screen_wide", picked.with_styles());
        }
    }
    terminal.send(Key::Char('S'))?;
    let save = terminal.wait_frame(|s| s.contains("Save filter") && s.cursor_visible())?;
    assert!(!is_active_label(&save, "Saved filters"));
    terminal.send_str("Cancelled")?;
    terminal.send(Key::Esc)?;
    let cancelled = terminal.wait_frame(|s| {
        !s.cursor_visible() && !s.contains("Save filter") && s.contains("Saved filters")
    })?;
    assert!(cancelled.contains("298 matching themes"));
    assert!(is_active_label(&cancelled, "Saved filters"));
    assert_eq!(fs::read_to_string(&path).unwrap(), disk_before);
    terminal.send(Key::Char('S'))?;
    terminal.wait_frame(|s| s.cursor_visible())?;
    terminal.send_str("Captured")?;
    terminal.send(Key::Enter)?;
    let saved = terminal.wait_frame(|s| {
        !s.cursor_visible()
            && s.contains("Saved filters")
            && s.contains("Captured")
            && s.contains("348 matching themes")
            && s.row_text(4).contains("4 / 4")
    })?;
    assert!(saved.contains("Selected filter"));
    assert_eq!(
        disk(&path)["saved_filters"][2]["query"].as_str(),
        Some("ignore:false")
    );
    terminal.send(Key::Char('e'))?;
    terminal.wait_frame(|s| s.contains("Rename filter") && s.cursor_visible())?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("Renamed")?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| !s.cursor_visible() && s.contains("Filter renamed"))?;
    terminal.send(Key::Esc)?;
    let back = terminal.wait_frame(|s| !s.contains("Saved filters") && s.contains("3024 Night"))?;
    assert_eq!(
        back.with_styles().to_string(),
        before.with_styles().to_string()
    );
    terminal.send_str("sj")?;
    terminal.wait_frame(|s| s.contains("Saved filters") && s.contains("298 matching themes"))?;
    terminal.send(Key::Enter)?;
    let applied = terminal.wait_frame(|s| {
        !s.contains("Saved filters")
            && s.row_text(0).contains(mode)
            && theme_list_visible(s) == list_visible
            && s.contains("298 / 348")
    })?;
    assert!(applied.contains("3024 Day"));
    quit(&mut terminal)
}
