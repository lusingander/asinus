mod support;

use rstest::rstest;
use support::{
    apply_query, quit, resize_and_wait,
    screen::{footer_contains, is_active_label, is_selected, theme_list_visible},
    wait_for,
};
use termlens::{Key, Screen};

fn panel(screen: &Screen, title: &str) -> bool {
    screen.contains(title) && !screen.contains("Help ·") && !screen.cursor_visible()
}

fn fills_main(screen: &Screen, title: &str) -> bool {
    panel(screen, title) && screen.row_text(4).contains(title)
}

#[rstest]
#[case("i", "Theme info · Overview", "Overview")]
#[case("d", "Color details", "Color details")]
#[case("pjczi", "Theme info · Overview", "Overview")]
#[case("pjczvi", "Theme info · Overview", "Overview")]
#[case("pjczd", "Color details", "Color details")]
#[case("pjczvd", "Color details", "Color details")]
fn maximized_panels_keep_focus_and_list_choice_through_resize_and_close(
    #[case] keys: &str,
    #[case] title: &str,
    #[case] focus_label: &str,
    #[values(false, true)] current_focus: bool,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(120, 48)?;
    support::wait_for_explore(&mut terminal)?;
    terminal.send_str(keys)?;
    wait_for(&mut terminal, |s| {
        panel(s, title) && theme_list_visible(s) && footer_contains(s, "Esc Close")
    })?;
    let colors = title == "Color details";
    if colors {
        terminal.send(Key::Char('G'))?;
        wait_for(&mut terminal, |s| {
            panel(s, title) && is_selected(s, "Selection")
        })?;
    }
    if current_focus {
        terminal.send(Key::Tab)?;
        wait_for(&mut terminal, |s| {
            panel(s, title) && !is_active_label(s, focus_label)
        })?;
    }
    terminal.send(Key::Char('Z'))?;
    let maximized = wait_for(&mut terminal, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    assert!(!theme_list_visible(&maximized));
    assert_eq!(is_active_label(&maximized, focus_label), !current_focus);
    if colors {
        assert_eq!(is_selected(&maximized, "Selection"), !current_focus);
        assert!(maximized.contains("Foreground"));
    } else {
        assert!(maximized.contains("Ratio"));
    }
    if !current_focus {
        if keys == "i" {
            insta::assert_snapshot!("info_maximized", maximized.with_styles());
        } else if keys == "pjczd" {
            insta::assert_snapshot!("colors_maximized", maximized.with_styles());
        }
    }
    terminal.send(Key::Char('z'))?;
    wait_for(&mut terminal, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    resize_and_wait(&mut terminal, 60, 22, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    let expanded = resize_and_wait(&mut terminal, 120, 48, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    assert_eq!(is_active_label(&expanded, focus_label), !current_focus);
    terminal.send(Key::Char('Z'))?;
    let restored = wait_for(&mut terminal, |s| {
        panel(s, title) && !fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    assert!(!theme_list_visible(&restored));
    terminal.send(Key::Char('z'))?;
    wait_for(&mut terminal, |s| panel(s, title) && theme_list_visible(s))?;
    terminal.send(Key::Char('Z'))?;
    wait_for(&mut terminal, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        !s.contains(title) && theme_list_visible(s)
    })?;
    terminal.send(Key::Char(if colors { 'd' } else { 'i' }))?;
    wait_for(&mut terminal, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    quit(&mut terminal)
}

#[test]
fn maximized_credits_keep_selection_through_search_and_panel_replacement() -> termlens::Result<()> {
    let title = "Theme info · Credits";
    let mut terminal = support::isolated(120, 48)?;
    support::wait_for_explore(&mut terminal)?;
    apply_query(&mut terminal, "Firewatch", 1)?;
    terminal.send_str("pcilG")?;
    wait_for(&mut terminal, |s| {
        panel(s, title) && is_selected(s, "Firewatch game")
    })?;
    terminal.send(Key::Char('Z'))?;
    wait_for(&mut terminal, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close") && is_selected(s, "Firewatch game")
    })?;
    terminal.send(Key::Char('/'))?;
    wait_for(&mut terminal, |s| {
        s.contains("Search · Draft") && s.cursor_visible()
    })?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close") && is_selected(s, "Firewatch game")
    })?;
    terminal.send(Key::Char('d'))?;
    wait_for(&mut terminal, |s| {
        fills_main(s, "Color details") && footer_contains(s, "Esc Close")
    })?;
    terminal.send(Key::Char('i'))?;
    wait_for(&mut terminal, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close") && is_selected(s, "Firewatch game")
    })?;
    terminal.send(Key::Char('d'))?;
    wait_for(&mut terminal, |s| {
        fills_main(s, "Color details") && footer_contains(s, "Esc Close")
    })?;
    terminal.send(Key::Char('Z'))?;
    wait_for(&mut terminal, |s| {
        panel(s, "Color details")
            && !fills_main(s, "Color details")
            && footer_contains(s, "Esc Close")
    })?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        panel(s, title)
            && !fills_main(s, title)
            && footer_contains(s, "Esc Close")
            && is_selected(s, "Firewatch game")
    })?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        !s.contains("Theme info ·") && !s.contains("Color details")
    })?;
    quit(&mut terminal)
}

#[rstest]
#[case("i", "Theme info · Overview", 27)]
#[case("pjczi", "Theme info · Overview", 27)]
#[case("pjczvi", "Theme info · Overview", 35)]
#[case("pjczd", "Color details", 25)]
#[case("pjczvd", "Color details", 33)]
fn maximize_choice_is_distinct_from_automatic_full_area_at_small_sizes(
    #[case] keys: &str,
    #[case] title: &str,
    #[case] preview_height: u16,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(60, 22)?;
    support::wait_for_explore(&mut terminal)?;
    terminal.send_str(keys)?;
    wait_for(&mut terminal, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    terminal.send(Key::Char('Z'))?;
    wait_for(&mut terminal, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    resize_and_wait(&mut terminal, 60, preview_height, |s| fills_main(s, title))?;
    terminal.send(Key::Char('Z'))?;
    wait_for(&mut terminal, |s| {
        panel(s, title) && !fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    resize_and_wait(&mut terminal, 60, preview_height - 1, |s| {
        fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    let docked = resize_and_wait(&mut terminal, 60, preview_height, |s| {
        panel(s, title) && !fills_main(s, title) && footer_contains(s, "Esc Close")
    })?;
    assert!(theme_list_visible(&docked));
    assert!(docked.contains("Foreground"));
    quit(&mut terminal)
}
