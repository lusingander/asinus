mod support;

use rstest::rstest;
use support::{
    apply_query, edit_query, quit, resize_and_wait,
    screen::{comparison_layout, is_active_label, theme_list_visible},
    wait_for,
};
use termlens::{Key, Screen};

fn colors(screen: &Screen, focus: &str) -> bool {
    screen.contains("Color details")
        && is_active_label(screen, "Color details") == (focus == "Colors")
        && !screen.cursor_visible()
        && !screen.contains("Help ·")
        && !screen.contains("· Draft")
        && !screen.contains("Select preview")
}

fn selected(screen: &Screen, role: &str, context: &str) -> bool {
    let Some((header_row, role_col)) = screen.find_all("Role").into_iter().find(|(row, _)| {
        let text = screen.row_text(*row);
        text.contains("Pin") || text.contains("Current")
    }) else {
        return false;
    };
    let Some((_, a_col)) = screen
        .find_all("Pin")
        .into_iter()
        .chain(screen.find_all("Current"))
        .filter(|(row, _)| *row == header_row)
        .min_by_key(|(_, col)| *col)
    else {
        return false;
    };
    let role_field = |row| {
        screen
            .row_text(row)
            .chars()
            .skip(role_col as usize)
            .take((a_col - role_col) as usize)
            .collect::<String>()
    };
    screen.find_all(context).into_iter().any(|(row, col)| {
        row > 0
            && col == role_col
            && role_field(row).trim_end() == context
            && role_field(row - 1).trim_end() == role
    })
}

#[rstest]
#[case(false)]
#[case(true)]
fn comparison_list_and_full_width_details_survive_resize_and_return(
    #[case] vertical: bool,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(120, 48)?;
    support::wait_for_explore(&mut terminal)?;
    terminal.send_str("pjzc")?;
    let compared = wait_for(&mut terminal, |s| {
        s.contains("Pin: 3024 Day") && s.contains("Current: 3024 Night")
    })?;
    assert!(!compared.row_text(4).contains("Themes"));
    if vertical {
        terminal.send(Key::Char('v'))?;
    }
    terminal.send(Key::Char('z'))?;
    wait_for(&mut terminal, |s| {
        s.row_text(4).contains("Themes") && comparison_layout(s, vertical)
    })?;
    terminal.send(Key::Char('d'))?;
    let docked = wait_for(&mut terminal, |s| {
        colors(s, "Colors") && s.row_text(4).contains("Themes")
    })?;
    if vertical {
        insta::assert_snapshot!("compare_list_vertical", docked.with_styles());
    } else {
        insta::assert_snapshot!("compare_list_horizontal", docked.with_styles());
    }
    terminal.send(Key::Char('j'))?;
    wait_for(&mut terminal, |s| {
        colors(s, "Colors") && selected(s, "Background", "Stored value")
    })?;
    terminal.send(Key::Char('z'))?;
    wait_for(&mut terminal, |s| {
        colors(s, "Colors") && !s.row_text(4).contains("Themes")
    })?;
    terminal.send(Key::Char('z'))?;
    wait_for(&mut terminal, |s| {
        colors(s, "Colors")
            && selected(s, "Background", "Stored value")
            && s.row_text(4).contains("Themes")
    })?;
    resize_and_wait(&mut terminal, 60, 22, |s| {
        colors(s, "Colors") && s.row_text(4).contains("Color details")
    })?;
    let restored = resize_and_wait(&mut terminal, 120, 48, |s| {
        colors(s, "Colors") && s.row_text(4).contains("Themes")
    })?;
    assert!(selected(&restored, "Background", "Stored value"));
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        !s.contains("Color details") && s.row_text(4).contains("Themes")
    })?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Explore") && !theme_list_visible(s)
    })?;
    terminal.send(Key::Char('c'))?;
    wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Compare") && s.row_text(4).contains("Themes")
    })?;
    quit(&mut terminal)
}

#[rstest]
#[case(false)]
#[case(true)]
fn colors_follow_the_current_theme_and_restore_search_and_explore_list_visibility(
    #[case] explore_list_hidden: bool,
) -> termlens::Result<()> {
    let state = tempfile::tempdir().unwrap();
    let marks = state.path().join("marks.toml");
    let workspace = state.path().join("workspace.toml");
    let (mut terminal, _) = support::start(80, 24, &marks, &workspace)?;
    terminal.send_str(if explore_list_hidden { "pjzcd" } else { "pjcd" })?;
    wait_for(&mut terminal, |s| {
        colors(s, "Colors") && s.row_text(4).contains("Pin:")
    })?;
    terminal.send_str("Gkkkkkkkkk")?;
    wait_for(&mut terminal, |s| {
        colors(s, "Colors") && selected(s, "Bright Black", "vs own BG")
    })?;
    terminal.send_str("mpx")?;
    terminal.send(Key::Tab)?;
    let screen = wait_for(&mut terminal, |s| colors(s, "Current"))?;
    assert!(screen.contains("3024 Day"));
    assert!(!marks.exists());
    assert!(!workspace.exists());
    terminal.send(Key::Char('j'))?;
    let moved = wait_for(&mut terminal, |s| {
        colors(s, "Current") && s.contains("Aardvark Blue")
    })?;
    assert!(selected(&moved, "Bright Black", "vs own BG"));
    assert!(moved.row_text(4).contains("Pin:"));
    terminal.send(Key::Tab)?;
    wait_for(&mut terminal, |s| colors(s, "Colors"))?;
    edit_query(&mut terminal, "missing-theme-e1")?;
    wait_for(&mut terminal, |s| {
        s.cursor_visible() && s.contains("0 / 348")
    })?;
    terminal.send(Key::Esc)?;
    let cancelled = wait_for(&mut terminal, |s| {
        colors(s, "Colors") && s.contains("Aardvark Blue")
    })?;
    assert!(selected(&cancelled, "Bright Black", "vs own BG"));
    let empty = apply_query(&mut terminal, "missing-theme-e1", 0)?;
    assert!(colors(&empty, "Colors"));
    assert!(empty.contains("No theme selected"));
    assert!(empty.contains("3024 Day"));
    terminal.send(Key::F(1))?;
    wait_for(&mut terminal, |s| s.contains("Help · Color details"))?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| colors(s, "Colors"))?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        !s.contains("Color details") && s.row_text(0).contains("Compare")
    })?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Explore") && theme_list_visible(s) != explore_list_hidden
    })?;
    quit(&mut terminal)
}

#[test]
fn colors_resize_without_switching_focus_and_keep_menus_and_filters_local() -> termlens::Result<()>
{
    let mut terminal = support::isolated(120, 36)?;
    support::wait_for_explore(&mut terminal)?;
    terminal.send_str("]pjcd")?;
    let wide = wait_for(&mut terminal, |s| {
        colors(s, "Colors") && s.row_text(4).contains("Pin:")
    })?;
    insta::assert_snapshot!("colors_docked", wide.with_styles());
    terminal.send(Key::Ctrl('f'))?;
    wait_for(&mut terminal, |s| selected(s, "Magenta", "vs own BG"))?;
    terminal.send(Key::Tab)?;
    wait_for(&mut terminal, |s| colors(s, "Current"))?;
    terminal.send(Key::Char('v'))?;
    wait_for(&mut terminal, |s| {
        colors(s, "Current") && comparison_layout(s, true)
    })?;
    resize_and_wait(&mut terminal, 80, 48, |s| {
        colors(s, "Current") && s.row_text(4).contains("Pin:")
    })?;
    resize_and_wait(&mut terminal, 59, 22, |s| s.contains("Terminal too small"))?;
    resize_and_wait(&mut terminal, 60, 22, |s| {
        colors(s, "Current") && s.row_text(4).contains("Color details")
    })?;
    terminal.send(Key::Tab)?;
    let minimum = wait_for(&mut terminal, |s| colors(s, "Colors"))?;
    insta::assert_snapshot!("colors_minimum", minimum.with_styles());
    for (key, marker) in [
        ('P', "Select preview"),
        ('f', "· Draft"),
        ('y', "Copy theme"),
        ('s', "Saved filters"),
    ] {
        terminal.send(Key::Char(key))?;
        wait_for(&mut terminal, |s| s.contains(marker))?;
        terminal.send(Key::Esc)?;
        let closed = wait_for(&mut terminal, |s| {
            colors(s, "Colors") && !s.contains(marker)
        })?;
        assert!(selected(&closed, "Magenta", "vs own BG"));
    }
    terminal.send(Key::Char('c'))?;
    wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Explore")
            && colors(s, "Colors")
            && selected(s, "Magenta", "vs own BG")
    })?;
    quit(&mut terminal)
}

#[test]
fn optional_colors_stay_unspecified_and_aliases_compare_equal() -> termlens::Result<()> {
    let mut terminal = support::isolated(80, 24)?;
    support::wait_for_explore(&mut terminal)?;
    apply_query(&mut terminal, "DraculaPlus", 1)?;
    terminal.send(Key::Char('p'))?;
    apply_query(&mut terminal, "Dracula", 3)?;
    terminal.send_str("gcdG")?;
    let missing = wait_for(&mut terminal, |s| {
        colors(s, "Colors") && selected(s, "Selection", "Stored value") && s.contains("Missing")
    })?;
    assert!(missing.contains("Not specified"));
    assert!(missing.contains("Dracula"));
    terminal.send(Key::Char('c'))?;
    wait_for(&mut terminal, |s| s.row_text(0).contains("Explore"))?;
    terminal.send(Key::Char('d'))?;
    wait_for(&mut terminal, |s| !s.contains("Color details"))?;
    apply_query(&mut terminal, "Catppuccin Mocha", 1)?;
    terminal.send(Key::Char('p'))?;
    apply_query(&mut terminal, "catppuccin-mocha", 1)?;
    terminal.send_str("cdg")?;
    wait_for(&mut terminal, |s| {
        colors(s, "Colors")
            && selected(s, "Foreground", "vs own BG")
            && s.contains("RGB Same 20 Diff 0 Missing 0")
    })?;
    quit(&mut terminal)
}

fn single_theme(screen: &Screen, target: &str, name: &str) -> bool {
    colors(screen, "Colors")
        && screen
            .find_all("Role")
            .into_iter()
            .any(|(row, _)| screen.row_text(row).contains(&format!("{target}: {name}")))
}

#[rstest]
#[case(60, 22)]
#[case(120, 36)]
fn explore_colors_switch_targets_and_keep_role_focus_and_layout_across_comparison(
    #[case] width: u16,
    #[case] height: u16,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(width, height)?;
    support::wait_for_explore(&mut terminal)?;
    terminal.send(Key::Char('d'))?;
    let single = wait_for(&mut terminal, |s| single_theme(s, "Current", "3024 Day"))?;
    assert!(!single.contains("RGB"));
    if width == 60 {
        insta::assert_snapshot!("explore_colors_minimum", single.with_styles());
    } else {
        insta::assert_snapshot!("explore_colors_docked", single.with_styles());
    }
    terminal.send(Key::F(1))?;
    let help = wait_for(&mut terminal, |s| s.contains("Help · Color details"))?;
    assert!(!help.contains("RGB: Same"));
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| single_theme(s, "Current", "3024 Day"))?;
    terminal.send_str("jpxm")?;
    terminal.send(Key::Tab)?;
    wait_for(&mut terminal, |s| {
        colors(s, "Current") && selected(s, "Background", "Stored value")
    })?;
    terminal.send_str("pj")?;
    wait_for(&mut terminal, |s| {
        colors(s, "Current") && s.contains("3024 Night")
    })?;
    terminal.send(Key::BackTab)?;
    wait_for(&mut terminal, |s| single_theme(s, "Current", "3024 Night"))?;
    edit_query(&mut terminal, "missing-theme-e1")?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| single_theme(s, "Current", "3024 Night"))?;
    terminal.send(Key::Char('t'))?;
    wait_for(&mut terminal, |s| single_theme(s, "Pin", "3024 Day"))?;
    terminal.send(Key::Char('j'))?;
    wait_for(&mut terminal, |s| selected(s, "Black", "vs own BG"))?;
    for (key, marker) in [
        ('f', "Filters · Draft"),
        ('y', "Copy theme"),
        ('s', "Saved filters"),
        ('i', "Theme info ·"),
    ] {
        terminal.send(Key::Char(key))?;
        wait_for(&mut terminal, |s| s.contains(marker))?;
        terminal.send(Key::Esc)?;
        wait_for(&mut terminal, |s| {
            single_theme(s, "Pin", "3024 Day") && selected(s, "Black", "vs own BG")
        })?;
    }
    terminal.send(Key::Char('Z'))?;
    wait_for(&mut terminal, |s| {
        colors(s, "Colors") && s.row_text(4).contains("Color details")
    })?;
    terminal.send(Key::Char('c'))?;
    let compared = wait_for(&mut terminal, |s| colors(s, "Colors") && s.contains("RGB"))?;
    terminal.send(Key::Char('t'))?;
    terminal.send(Key::F(1))?;
    wait_for(&mut terminal, |s| s.contains("Help · Color details"))?;
    terminal.send(Key::Esc)?;
    let after = wait_for(&mut terminal, |s| colors(s, "Colors"))?;
    assert_eq!(
        after.with_styles().to_string(),
        compared.with_styles().to_string()
    );
    terminal.send(Key::Char('c'))?;
    wait_for(&mut terminal, |s| {
        single_theme(s, "Pin", "3024 Day") && s.row_text(0).contains("Explore")
    })?;
    terminal.send(Key::Char('d'))?;
    wait_for(&mut terminal, |s| !s.contains("Color details"))?;
    terminal.send(Key::Char('d'))?;
    wait_for(&mut terminal, |s| {
        single_theme(s, "Current", "3024 Night") && selected(s, "Black", "vs own BG")
    })?;
    terminal.send(Key::Char('t'))?;
    wait_for(&mut terminal, |s| single_theme(s, "Pin", "3024 Day"))?;
    terminal.send(Key::Tab)?;
    terminal.send_str("kp")?;
    wait_for(&mut terminal, |s| {
        colors(s, "Current") && !s.row_text(0).contains("Pin:")
    })?;
    terminal.send(Key::Tab)?;
    wait_for(&mut terminal, |s| {
        single_theme(s, "Current", "3024 Day") && selected(s, "Black", "vs own BG")
    })?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        !s.contains("Color details") && theme_list_visible(s)
    })?;
    terminal.send(Key::Char('d'))?;
    wait_for(&mut terminal, |s| {
        single_theme(s, "Current", "3024 Day")
            && s.row_text(4).contains("Color details")
            && selected(s, "Black", "vs own BG")
    })?;
    quit(&mut terminal)
}

#[test]
fn explore_colors_handle_missing_values_empty_results_and_pin_fallback() -> termlens::Result<()> {
    let mut terminal = support::isolated(80, 24)?;
    support::wait_for_explore(&mut terminal)?;
    apply_query(&mut terminal, "DraculaPlus", 1)?;
    terminal.send_str("dG")?;
    let missing = wait_for(&mut terminal, |s| {
        single_theme(s, "Current", "DraculaPlus") && selected(s, "Selection", "Stored value")
    })?;
    assert!(missing.contains("Not specified"));
    assert!(!missing.contains("Missing"));
    let empty = apply_query(&mut terminal, "missing-theme-e1", 0)?;
    assert!(colors(&empty, "Colors"));
    assert!(empty.contains("No theme selected"));
    assert!(!empty.contains("Not specified"));
    terminal.send(Key::Char('d'))?;
    wait_for(&mut terminal, |s| !s.contains("Color details"))?;
    terminal.send(Key::Char('d'))?;
    wait_for(&mut terminal, |s| {
        !s.contains("Color details") && s.contains("No theme to inspect")
    })?;
    apply_query(&mut terminal, "DraculaPlus", 1)?;
    terminal.send_str("pd")?;
    wait_for(&mut terminal, |s| single_theme(s, "Current", "DraculaPlus"))?;
    let fallback = apply_query(&mut terminal, "missing-theme-e1", 0)?;
    assert!(single_theme(&fallback, "Pin", "DraculaPlus"));
    terminal.send(Key::Char('/'))?;
    wait_for(&mut terminal, |s| s.contains("Search · Draft"))?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("DraculaPlus")?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        single_theme(s, "Pin", "DraculaPlus") && s.contains("0 / 348")
    })?;
    terminal.send(Key::Char('d'))?;
    wait_for(&mut terminal, |s| !s.contains("Color details"))?;
    terminal.send(Key::Char('d'))?;
    wait_for(&mut terminal, |s| single_theme(s, "Pin", "DraculaPlus"))?;
    quit(&mut terminal)
}
