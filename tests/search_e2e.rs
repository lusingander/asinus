mod support;

use rstest::rstest;
use support::{
    edit_query as search, quit, resize_and_wait,
    screen::{
        comparing, comparison_layout, footer_contains, matching_themes, shows_current_theme,
        theme_list_visible,
    },
};
use termlens::{Color, Key, Screen, Terminal};

fn start() -> termlens::Result<support::IsolatedTerminal> {
    let mut terminal = support::isolated(80, 24)?;
    support::wait_for_explore(&mut terminal)?;
    Ok(terminal)
}

fn results(terminal: &mut Terminal, query: &str, count: usize) -> termlens::Result<Screen> {
    terminal.wait_frame(|s| {
        s.contains("Search · Draft")
            && s.contains(&format!("/ {query}"))
            && matching_themes(s, count)
    })
}

fn filters(terminal: &mut Terminal, query: &str, count: usize) -> termlens::Result<Screen> {
    terminal.wait_frame(|s| {
        let terms: Vec<&str> = query
            .split_whitespace()
            .filter_map(|part| match part {
                "ignore:false" => None,
                "type:dark" => Some("Dark"),
                "type:light" => Some("Light"),
                "favorite:true" => Some("Favorites"),
                "favorite:false" => Some("Not favorite"),
                "ignore:all" => Some("Ignored included"),
                "ignore:true" => Some("Ignored only"),
                _ => Some(part),
            })
            .collect();
        terms.iter().all(|term| s.row_text(1).contains(term))
            && matching_themes(s, count)
            && s.contains("Filters · Draft")
    })
}

fn clear_draft(terminal: &mut Terminal) -> termlens::Result<()> {
    terminal.send(Key::BackTab)?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send(Key::Tab)?;
    terminal.send(Key::Char('l'))?;
    filters(terminal, "", 348)?;
    Ok(())
}

#[test]
fn live_search_highlights_matches_and_keeps_text_keys_literal() -> termlens::Result<()> {
    let mut terminal = start()?;
    search(&mut terminal, "catpp")?;
    let screen = results(&mut terminal, "catpp", 8)?;
    insta::assert_snapshot!("search_minimum", screen.with_styles());
    assert!(screen.cursor_visible());
    assert!(screen.find_all("Catppuccin Frappe").contains(&(8, 31)));
    for (name, selected) in [("Catppuccin Frappe", true), ("Catppuccin Latte", false)] {
        let (y, x) = screen
            .find_all(name)
            .into_iter()
            .find(|(y, x)| *x < 31 && *y >= 10)
            .unwrap();
        for column in x..x + 5 {
            let style = screen.cell(y, column).unwrap().style();
            assert_eq!(
                style.fg,
                if selected {
                    Color::Default
                } else {
                    Color::Indexed(3)
                }
            );
            assert_eq!(style.bg, Color::Default);
            assert_eq!(style.bold, !selected);
            assert_eq!(style.reverse, selected);
        }
        assert_eq!(screen.cell(y, x + 5).unwrap().style().fg, Color::Default);
    }
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| !s.cursor_visible() && matching_themes(s, 8))?;
    terminal.send_str("/")?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("catppRjpkcv")?;
    let text = results(&mut terminal, "catppRjpkcv", 0)?;
    assert!(text.contains("No theme selected"));
    assert!(text.contains("No matching themes"));
    assert!(!text.contains("Pin a theme first"));
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !s.cursor_visible() && s.contains("8 / 348"))?;
    terminal.send(Key::Char('r'))?;
    terminal.send(Key::Char('f'))?;
    filters(&mut terminal, "catpp", 8)?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !s.contains("· Draft"))?;
    terminal.send(Key::Char('R'))?;
    let reset = terminal.wait_frame(|s| matching_themes(s, 348) && !s.cursor_visible())?;
    assert!(reset.find_all("Catppuccin Frappe").contains(&(4, 31)));
    quit(&mut terminal)
}

#[rstest]
#[case("Catpp", 4)]
#[case("mocha catpp", 2)]
#[case("catpp type:dark", 6)]
#[case("catpp type:light", 2)]
#[case("\"Solarized Dark\"", 4)]
fn query_conditions_work_through_terminal_input(
    #[case] query: &str,
    #[case] count: usize,
) -> termlens::Result<()> {
    let mut terminal = start()?;
    terminal.send(Key::Char('/'))?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str(query)?;
    results(&mut terminal, query, count)?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| !s.cursor_visible() && matching_themes(s, count))?;
    terminal.send(Key::Char('R'))?;
    terminal.wait_frame(|s| matching_themes(s, 348) && !s.cursor_visible())?;
    quit(&mut terminal)
}

#[rstest]
#[rustfmt::skip]
#[case("ignore:false", 348, "catpp", "catpp ignore:false", 8)]
#[case("type:light ignore:all", 50, "catpp", "catpp type:light ignore:all", 2)]
#[case("catpp type:dark ignore:all", 6, " mocha", "catpp mocha type:dark ignore:all", 2)]
fn search_starts_at_the_name_end_and_keeps_filter_conditions(
    #[case] query: &str,
    #[case] count: usize,
    #[case] text: &str,
    #[case] expected: &str,
    #[case] expected_count: usize,
) -> termlens::Result<()> {
    let mut terminal = start()?;
    support::apply_query(&mut terminal, query, count)?;
    terminal.send(Key::Char('/'))?;
    terminal.send_str(text)?;
    let screen = results(&mut terminal, expected, expected_count)?;
    assert!(screen.cursor_visible());
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| !s.cursor_visible() && matching_themes(s, expected_count))?;
    quit(&mut terminal)
}

#[rstest]
#[case(false)]
#[case(true)]
fn query_cancel_and_zero_results_preserve_comparison(
    #[case] vertical: bool,
) -> termlens::Result<()> {
    let mut terminal = start()?;
    terminal.send_str("Gpc")?;
    if vertical {
        terminal.send(Key::Char('v'))?;
    }
    let before = terminal.wait_frame(|s| {
        comparing(s) && shows_current_theme(s, "zenwritten_light") && comparison_layout(s, vertical)
    })?;
    search(&mut terminal, "catpp type:")?;
    let invalid =
        terminal.wait_frame(|s| s.contains("Last valid results") && s.contains("0 / 348"))?;
    assert!(invalid.contains("type: choose dark or light"));
    if !vertical {
        insta::assert_snapshot!("invalid_search_minimum", invalid.with_styles());
    }
    resize_and_wait(&mut terminal, 100, 30, |s| {
        s.contains("Last valid results")
            && s.row_text(1).contains("catpp type")
            && footer_contains(s, "Esc Cancel")
    })?;
    resize_and_wait(&mut terminal, 80, 24, |s| {
        s.contains("Last valid results") && footer_contains(s, "Esc Cancel")
    })?;
    terminal.send(Key::Enter)?;
    terminal.send(Key::Char('d'))?;
    terminal.wait_frame(|s| s.contains("/ catpp type:d") && s.contains("Last valid results"))?;
    terminal.send(Key::Esc)?;
    let restored = terminal.wait_frame(|s| {
        comparing(s) && shows_current_theme(s, "zenwritten_light") && !s.cursor_visible()
    })?;
    assert_eq!(
        restored.with_styles().to_string(),
        before.with_styles().to_string()
    );

    search(&mut terminal, "no-such-theme")?;
    results(&mut terminal, "no-such-theme", 0)?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| comparing(s) && !s.cursor_visible())?;
    terminal.send_str("jp")?;
    let empty = terminal.wait_frame(|s| {
        s.contains("Current") && s.contains("No theme selected") && s.contains("0 / 348")
    })?;
    let pin_title = "zenwritten_light";
    assert!(empty.row_text(4).contains(&format!("Pin: {pin_title}")));
    assert!(!empty.contains("*zenwritten_light"));
    assert!(comparing(&empty));
    terminal.send(Key::Char('/'))?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("catpp mocha")?;
    results(&mut terminal, "catpp mocha", 2)?;
    terminal.send(Key::Enter)?;
    let recovered =
        terminal.wait_frame(|s| comparing(s) && s.contains("2 / 348") && !s.cursor_visible())?;
    assert!(recovered.row_text(4).contains(&format!("Pin: {pin_title}")));
    assert!(shows_current_theme(&recovered, "Catppuccin Mocha"));
    terminal.send(Key::Char('R'))?;
    terminal
        .wait_frame(|s| matching_themes(s, 348) && shows_current_theme(s, "Catppuccin Mocha"))?;
    quit(&mut terminal)
}

#[test]
fn filters_preview_drafts_and_page_results_without_changing_the_current_theme()
-> termlens::Result<()> {
    let mut terminal = start()?;
    terminal.send_str(&"j".repeat(49))?;
    terminal.send_str("pcvf")?;
    filters(&mut terminal, "", 348)?;
    terminal.send(Key::BackTab)?;
    terminal.send_str("catpp")?;
    let all = filters(&mut terminal, "catpp", 8)?;
    assert!(all.contains("*  Catppuccin Mocha"));
    assert!(all.cursor_visible());
    assert!(all.row_text(23).contains("Enter Apply") && all.row_text(23).contains("Esc Cancel"));
    terminal.send(Key::Tab)?;
    terminal.send(Key::Char('l'))?;
    filters(&mut terminal, "catpp type:dark", 6)?;
    terminal.send(Key::Right)?;
    let light = filters(&mut terminal, "catpp type:light", 2)?;
    assert!(!light.cursor_visible());
    assert!(!light.contains("*  Catppuccin Mocha"));
    assert!(light.row_text(0).contains("Preview  Palette"));
    assert_eq!(
        light.row_text(23).split_whitespace().collect::<Vec<_>>(),
        "Tab/S-Tab Field  h/l Value  Enter Apply  Esc Cancel  C-c Quit  F1 Help"
            .split_whitespace()
            .collect::<Vec<_>>()
    );
    for y in 5..21 {
        for x in 2..24 {
            assert_eq!(light.cell(y, x).unwrap().style().bg, Color::Default);
        }
    }
    insta::assert_snapshot!("filters_light_minimum", light.with_styles());
    terminal.send(Key::Enter)?;
    let applied =
        terminal.wait_frame(|s| comparing(s) && s.contains("2 / 348") && !s.contains("· Draft"))?;
    assert!(applied.row_text(4).contains("Pin: Catppuccin Mocha"));
    assert!(applied.row_text(13).contains("Current: Catppuccin Latte"));
    terminal.send(Key::Char('f'))?;
    filters(&mut terminal, "catpp type:light", 2)?;
    clear_draft(&mut terminal)?;
    terminal.send(Key::Ctrl('f'))?;
    let paged = terminal.wait_frame(|s| s.row_text(6).contains("Aurora") && s.contains("All"))?;
    assert!(paged.row_text(21).contains("Bright Lights"));
    assert!(paged.row_text(1).contains("348 themes"));
    assert_eq!(
        paged
            .row_text(22)
            .chars()
            .take(25)
            .collect::<String>()
            .trim(),
        ""
    );
    assert!(paged.row_text(0).contains("Preview  Palette"));
    terminal.send(Key::Ctrl('b'))?;
    terminal.wait_frame(|s| s.row_text(6).contains("3024 Day"))?;
    terminal.send(Key::Esc)?;
    let cancelled =
        terminal.wait_frame(|s| comparing(s) && s.contains("2 / 348") && !s.contains("· Draft"))?;
    assert_eq!(
        cancelled.with_styles().to_string(),
        applied.with_styles().to_string()
    );
    terminal.send(Key::Char('f'))?;
    filters(&mut terminal, "catpp type:light", 2)?;
    clear_draft(&mut terminal)?;
    terminal.send(Key::Ctrl('f'))?;
    terminal.wait_frame(|s| s.row_text(6).contains("Aurora"))?;
    terminal.send(Key::Enter)?;
    let preserved = terminal.wait_frame(|s| matching_themes(s, 348) && comparing(s))?;
    assert!(preserved.row_text(4).contains("Pin: Catppuccin Mocha"));
    assert!(preserved.row_text(13).contains("Current: Catppuccin Latte"));
    quit(&mut terminal)
}

#[test]
fn filters_keep_the_last_valid_draft_and_distinguish_zero_after_resize() -> termlens::Result<()> {
    let mut terminal = start()?;
    terminal.send(Key::Char('f'))?;
    filters(&mut terminal, "", 348)?;
    terminal.send(Key::BackTab)?;
    terminal.send_str("\"catpp")?;
    let invalid = terminal
        .wait_frame(|s| s.contains("Last valid results") && s.contains("Close the double quote"))?;
    assert!(shows_current_theme(&invalid, "3024 Day"));
    assert!(invalid.row_text(1).contains("348"));
    terminal.send(Key::Enter)?;
    terminal.send(Key::Char('R'))?;
    terminal.wait_frame(|s| s.contains("\"catppR"))?;
    resize_and_wait(&mut terminal, 59, 22, |s| s.contains("Current:  59x22"))?;
    resize_and_wait(&mut terminal, 100, 30, |s| {
        s.contains("Last valid results")
            && s.contains("\"catppR")
            && footer_contains(s, "Esc Cancel")
    })?;
    let restored = resize_and_wait(&mut terminal, 80, 24, |s| {
        s.contains("Close the double quote")
            && s.cursor_visible()
            && footer_contains(s, "Esc Cancel")
    })?;
    assert!(restored.contains("\"catppR"));
    terminal.send(Key::Char('"'))?;
    let zero = filters(&mut terminal, "catppR", 0)?;
    assert!(zero.contains("No matching themes"));
    assert!(!zero.contains("Last valid results"));
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("No theme selected") && !s.contains("· Draft"))?;
    terminal.send(Key::Char('R'))?;
    terminal.wait_frame(|s| matching_themes(s, 348) && s.row_text(4).contains("3024 Day"))?;
    quit(&mut terminal)
}

#[rstest]
#[case(false, false)]
#[case(false, true)]
#[case(true, false)]
#[case(true, true)]
fn wide_editors_preview_selected_drafts_and_restore_explore_list_visibility(
    #[case] explore_list_hidden: bool,
    #[case] vertical: bool,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(120, 36)?;
    terminal.wait_frame(|s| matching_themes(s, 348) && footer_contains(s, "Quit"))?;
    terminal.send_str(&"j".repeat(49))?;
    terminal.send(Key::Char('p'))?;
    if explore_list_hidden {
        terminal.send(Key::Char('z'))?;
    }
    terminal.send(Key::Char('c'))?;
    if vertical {
        terminal.send(Key::Char('v'))?;
    }
    let before = terminal.wait_frame(|s| {
        comparing(s) && shows_current_theme(s, "Catppuccin Mocha") && comparison_layout(s, vertical)
    })?;

    for cancel in [true, false] {
        terminal.send(Key::Char('f'))?;
        filters(&mut terminal, "", 348)?;
        terminal.send(Key::BackTab)?;
        terminal.send_str("catpp")?;
        terminal.send(Key::Tab)?;
        terminal.send_str("ll")?;
        let light = filters(&mut terminal, "catpp type:light", 2)?;
        assert!(light.contains("•    Catppuccin Latte"));
        assert!(light.contains("Pin: Catppuccin Mocha"));
        assert!(light.contains("Current: Catppuccin Latte"));
        if !explore_list_hidden && !cancel {
            if vertical {
                insta::assert_snapshot!("filters_vertical_wide", light.with_styles());
            } else {
                insta::assert_snapshot!("filters_horizontal_wide", light.with_styles());
            }
        }
        terminal.send(Key::Char('l'))?;
        let all = filters(&mut terminal, "catpp", 8)?;
        assert!(all.contains("Current: Catppuccin Latte"));
        terminal.send(if cancel { Key::Esc } else { Key::Enter })?;
        let closed = terminal.wait_frame(|s| {
            comparing(s)
                && shows_current_theme(
                    s,
                    if cancel {
                        "Catppuccin Mocha"
                    } else {
                        "Catppuccin Latte"
                    },
                )
        })?;
        if cancel {
            assert_eq!(
                closed.with_styles().to_string(),
                before.with_styles().to_string()
            );
        } else {
            assert!(closed.contains("Current: Catppuccin Latte"));
        }
    }
    search(&mut terminal, "catpp")?;
    results(&mut terminal, "catpp", 8)?;
    terminal.send(Key::Down)?;
    terminal.wait_frame(|s| shows_current_theme(s, "Catppuccin Macchiato"))?;
    terminal.send(Key::Char(':'))?;
    let invalid = terminal
        .wait_frame(|s| s.contains("Last valid results") && s.contains("Unknown filter:"))?;
    assert!(shows_current_theme(&invalid, "Catppuccin Macchiato"));
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.cursor_visible() && s.contains("Last valid results"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| comparing(s) && shows_current_theme(s, "Catppuccin Latte"))?;
    search(&mut terminal, "catpp")?;
    results(&mut terminal, "catpp", 8)?;
    terminal.send(Key::Down)?;
    terminal.wait_frame(|s| shows_current_theme(s, "Catppuccin Macchiato"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| comparing(s) && shows_current_theme(s, "Catppuccin Macchiato"))?;
    terminal.send(Key::Esc)?;
    let explored = terminal.wait_frame(|s| {
        s.row_text(0).contains("Explore")
            && theme_list_visible(s) != explore_list_hidden
            && shows_current_theme(s, "Catppuccin Macchiato")
    })?;
    assert!(explored.contains("Catppuccin Macchiato"));
    quit(&mut terminal)
}

#[test]
fn filters_page_without_moving_or_losing_the_current_theme_and_cancel_restores_the_viewport()
-> termlens::Result<()> {
    let mut terminal = start()?;
    terminal.send_str(&"j".repeat(49))?;
    let before = terminal.wait_frame(|s| shows_current_theme(s, "Catppuccin Mocha"))?;
    terminal.send(Key::Char('f'))?;
    let opened = filters(&mut terminal, "", 348)?;
    assert!(shows_current_theme(&opened, "Catppuccin Mocha"));
    let result_row = opened.find("Matches").unwrap().0 + 2;
    let first_result = opened.row_text(result_row);
    terminal.send(Key::Ctrl('b'))?;
    let paged = terminal.wait_frame(|s| {
        s.row_text(result_row) != first_result && shows_current_theme(s, "Catppuccin Mocha")
    })?;
    assert!(paged.contains("348 themes"));
    terminal.send(Key::BackTab)?;
    terminal.send(Key::Char('"'))?;
    let invalid = terminal
        .wait_frame(|s| s.contains("Last valid results") && s.contains("Close the double quote"))?;
    for y in result_row..22 {
        assert_eq!(
            invalid.row_text(y).chars().take(29).collect::<String>(),
            paged.row_text(y).chars().take(29).collect::<String>()
        );
    }
    terminal.send(Key::Enter)?;
    terminal.wait_frame(|s| s.contains("Last valid results") && s.cursor_visible())?;
    terminal.send(Key::Esc)?;
    let cancelled = terminal
        .wait_frame(|s| !s.contains("· Draft") && shows_current_theme(s, "Catppuccin Mocha"))?;
    assert_eq!(
        cancelled.with_styles().to_string(),
        before.with_styles().to_string()
    );
    terminal.send(Key::Char('f'))?;
    filters(&mut terminal, "", 348)?;
    terminal.send(Key::Ctrl('b'))?;
    terminal.wait_frame(|s| {
        s.row_text(result_row) != first_result && shows_current_theme(s, "Catppuccin Mocha")
    })?;
    terminal.send(Key::Enter)?;
    let applied = terminal
        .wait_frame(|s| !s.contains("· Draft") && shows_current_theme(s, "Catppuccin Mocha"))?;
    assert_eq!(
        applied.with_styles().to_string(),
        before.with_styles().to_string()
    );
    quit(&mut terminal)
}
