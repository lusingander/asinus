mod support;

use std::fs;

use rstest::rstest;
use support::{
    apply_query, edit_query, quit, resize_and_wait,
    screen::{
        comparing, focused_field_value, is_selected, matching_themes, shows_current_theme,
        theme_list_visible,
    },
    wait_for,
};
use termlens::{Key, Screen, Terminal};

const BLUE: &str = "catpp brightness:very-dark bg:blue contrast:medium";

fn field(
    terminal: &mut Terminal,
    label: &str,
    value: &str,
    count: usize,
) -> termlens::Result<Screen> {
    wait_for(terminal, |s| {
        s.contains("Filters · Draft")
            && focused_field_value(s, label, value)
            && matching_themes(s, count)
    })
}

#[rstest]
#[case(60, 22, false, false)]
#[case(80, 24, true, true)]
#[case(120, 36, false, true)]
#[case(120, 36, true, false)]
fn color_drafts_round_trip_and_restore_comparison(
    #[case] width: u16,
    #[case] height: u16,
    #[case] vertical: bool,
    #[case] explore_list_hidden: bool,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(width, height)?;
    support::wait_for_explore(&mut terminal)?;
    apply_query(&mut terminal, "Catppuccin Mocha", 1)?;
    terminal.send(Key::Char('p'))?;
    apply_query(&mut terminal, "catpp", 8)?;
    if explore_list_hidden {
        terminal.send(Key::Char('z'))?;
    }
    terminal.send(Key::Char('c'))?;
    if vertical {
        terminal.send(Key::Char('v'))?;
    }
    let before = wait_for(&mut terminal, |s| {
        comparing(s)
            && shows_current_theme(s, "Catppuccin Mocha")
            && support::screen::comparison_layout(s, vertical)
    })?;
    terminal.send(Key::Char('f'))?;
    field(&mut terminal, "Appearance", "All", 8)?;
    for _ in 0..3 {
        terminal.send(Key::Tab)?;
    }
    terminal.send(Key::Right)?;
    field(&mut terminal, "BG brightness", "Very dark", 2)?;
    terminal.send(Key::Tab)?;
    terminal.send(Key::Left)?;
    let empty = field(&mut terminal, "BG color", "Purple", 0)?;
    assert!(empty.contains("No theme selected") && empty.contains("No matching themes"));
    assert!(!empty.contains("Last valid results"));
    terminal.send(Key::Left)?;
    field(&mut terminal, "BG color", "Blue", 2)?;
    terminal.send(Key::Tab)?;
    terminal.send(Key::Right)?;
    terminal.send(Key::Right)?;
    field(&mut terminal, "Text contrast", "Medium", 2)?;
    resize_and_wait(&mut terminal, 59, 22, |s| s.contains("Terminal too small"))?;
    let draft = resize_and_wait(&mut terminal, width, height, |s| {
        focused_field_value(s, "Text contrast", "Medium") && matching_themes(s, 2)
    })?;
    assert!(draft.contains("Very dark") && draft.contains("Blue"));
    terminal.send(Key::Esc)?;
    let cancelled = wait_for(&mut terminal, comparing)?;
    assert_eq!(
        cancelled.with_styles().to_string(),
        before.with_styles().to_string()
    );

    apply_query(&mut terminal, BLUE, 2)?;
    terminal.send(Key::Char('f'))?;
    field(&mut terminal, "Appearance", "All", 2)?;
    for _ in 0..3 {
        terminal.send(Key::Tab)?;
    }
    field(&mut terminal, "BG brightness", "Very dark", 2)?;
    terminal.send(Key::Tab)?;
    field(&mut terminal, "BG color", "Blue", 2)?;
    terminal.send(Key::Right)?;
    field(&mut terminal, "BG color", "Purple", 0)?;
    terminal.send(Key::Enter)?;
    let applied = wait_for(&mut terminal, |s| comparing(s) && matching_themes(s, 0))?;
    assert!(applied.contains("No theme selected"));
    assert!(support::screen::preview_name_matches(
        &applied,
        "Pin:",
        "Catppuccin Mocha"
    ));
    apply_query(&mut terminal, BLUE, 2)?;
    terminal.send(Key::Esc)?;
    let restored = wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Explore")
            && theme_list_visible(s) != explore_list_hidden
            && shows_current_theme(s, "Catppuccin Mocha")
    })?;
    assert!(matching_themes(&restored, 2));
    quit(&mut terminal)
}

fn save(terminal: &mut Terminal, name: &str) -> termlens::Result<()> {
    terminal.send(Key::Char('S'))?;
    wait_for(terminal, |s| {
        s.contains("Save filter") && s.cursor_visible()
    })?;
    terminal.send_str(name)?;
    terminal.send(Key::Enter)?;
    wait_for(terminal, |s| {
        !s.cursor_visible() && s.contains("Filter saved")
    })?;
    Ok(())
}

#[test]
fn saved_color_conditions_restore_all_fields_and_valid_zero_results() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.toml");
    let marks = dir.path().join("marks.toml");
    let (mut terminal, _) = support::start(60, 22, &marks, &path)?;
    apply_query(&mut terminal, BLUE, 2)?;
    terminal.send(Key::Char('m'))?;
    wait_for(&mut terminal, |s| s.contains("Favorite saved"))?;
    let query =
        "catpp type:dark favorite:true ignore:false brightness:very-dark bg:blue contrast:medium";
    apply_query(&mut terminal, query, 1)?;
    save(&mut terminal, "Blue night")?;
    let saved: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(saved["saved_filters"][0]["query"].as_str(), Some(query));
    assert_eq!(saved["session"]["query"].as_str(), Some(query));

    terminal.send(Key::Char('f'))?;
    field(&mut terminal, "Appearance", "Dark", 1)?;
    for _ in 0..4 {
        terminal.send(Key::Tab)?;
    }
    terminal.send(Key::Right)?;
    field(&mut terminal, "BG color", "Purple", 0)?;
    terminal.send(Key::Enter)?;
    wait_for(&mut terminal, |s| {
        !s.contains("· Draft") && matching_themes(s, 0)
    })?;
    save(&mut terminal, "Empty")?;
    quit(&mut terminal)?;

    let (mut terminal, restored) = support::start(60, 22, &marks, &path)?;
    assert!(matching_themes(&restored, 0));
    terminal.send(Key::Char('f'))?;
    let restored = field(&mut terminal, "Appearance", "Dark", 0)?;
    assert!(
        restored.contains("Very dark")
            && restored.contains("Purple")
            && restored.contains("Medium")
    );
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| !s.contains("· Draft"))?;
    terminal.send(Key::Char('s'))?;
    wait_for(&mut terminal, |s| s.contains("Saved filters"))?;
    terminal.send(Key::Char('g'))?;
    wait_for(&mut terminal, |s| is_selected(s, "Initial"))?;
    terminal.send(Key::Char('j'))?;
    wait_for(&mut terminal, |s| {
        is_selected(s, "Blue night") && s.contains("1 matching theme")
    })?;
    terminal.send(Key::Enter)?;
    wait_for(&mut terminal, |s| {
        !s.contains("Saved filters") && matching_themes(s, 1)
    })?;
    let before = fs::read_to_string(&path).unwrap();
    edit_query(&mut terminal, "bg:purple contrast:unknown")?;
    wait_for(&mut terminal, |s| {
        s.contains("Last valid results") && s.contains("contrast: invalid value")
    })?;
    terminal.send(Key::Enter)?;
    terminal.send(Key::Char('!'))?;
    wait_for(&mut terminal, |s| {
        s.cursor_visible() && s.contains("unknown!")
    })?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        !s.contains("· Draft") && matching_themes(s, 1)
    })?;
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    quit(&mut terminal)?;

    let (mut terminal, restored) = support::start(60, 22, &marks, &path)?;
    assert!(matching_themes(&restored, 1));
    terminal.send(Key::Char('f'))?;
    let restored = field(&mut terminal, "Appearance", "Dark", 1)?;
    terminal.send(Key::Tab)?;
    field(&mut terminal, "Favorite", "Only", 1)?;
    terminal.send(Key::Tab)?;
    field(&mut terminal, "Ignored", "Hide", 1)?;
    assert!(
        restored.contains("Very dark") && restored.contains("Blue") && restored.contains("Medium")
    );
    quit(&mut terminal)
}
