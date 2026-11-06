mod support;

use std::fs;

use rstest::rstest;
use support::{
    apply_query, quit, resize_and_wait,
    screen::{
        comparison_layout, footer_contains, is_active_label, is_selected, theme_list_visible,
    },
    wait_for,
};
use termlens::{Key, Screen};

fn panel(screen: &Screen) -> bool {
    screen.contains("Theme info ·") && !screen.contains("Help ·") && !screen.contains("Copy theme")
}

fn credits(screen: &Screen, target: &str, name: &str) -> bool {
    panel(screen)
        && screen
            .find("Theme info · Credits")
            .is_some_and(|(row, _)| screen.row_text(row + 2).trim() == format!("{target}: {name}"))
}

fn info_focused(screen: &Screen) -> bool {
    panel(screen) && (is_active_label(screen, "Overview") || is_active_label(screen, "Credits"))
}

fn current_focused(screen: &Screen) -> bool {
    panel(screen) && !info_focused(screen)
}

#[rstest]
#[case("", "Explore")]
#[case("z", "Explore")]
#[case("d", "Explore")]
#[case("zd", "Explore")]
#[case("c", "Compare")]
#[case("cv", "Compare")]
#[case("cd", "Compare")]
#[case("cvd", "Compare")]
fn info_switches_sections_targets_and_focus_and_restores_the_original_view(
    #[case] keys: &str,
    #[case] mode: &str,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let marks = dir.path().join("marks.toml");
    let workspace = dir.path().join("workspace.toml");
    let (mut terminal, _) = support::start(80, 24, &marks, &workspace)?;
    terminal.send(Key::Char('p'))?;
    let explored = apply_query(&mut terminal, "Andromeda", 1)?;
    let before = if keys.is_empty() {
        explored
    } else {
        terminal.send_str(keys)?;
        terminal.wait_frame(|s| {
            s.row_text(0).contains(mode)
                && if keys.ends_with('d') {
                    footer_contains(s, "j/k Roles")
                } else if mode == "Compare" {
                    comparison_layout(s, keys.ends_with('v'))
                } else {
                    theme_list_visible(s) == keys.is_empty() && s.row_text(4).contains("Andromeda")
                }
        })?
    };
    let saved = fs::read(&workspace).unwrap();
    terminal.send(Key::Char('i'))?;
    terminal.wait_frame(|s| panel(s) && s.contains("Appearance") && !s.contains("memije"))?;
    terminal.send(Key::Char('l'))?;
    terminal.wait_frame(|s| credits(s, "Current", "Andromeda") && is_selected(s, "memije"))?;
    terminal.send(Key::Char('j'))?;
    let info_before = terminal.wait_frame(|s| is_selected(s, "EliverLara"))?;
    for (key, marker) in [
        ('/', "Search · Draft"),
        ('f', "Filters · Draft"),
        ('P', "Select preview"),
        ('y', "Copy theme"),
        ('S', "Save filter"),
        ('s', "Saved filters"),
    ] {
        terminal.send(Key::Char(key))?;
        terminal.wait_frame(|s| s.contains(marker))?;
        terminal.send(Key::Esc)?;
        let restored = terminal.wait_frame(|s| {
            info_focused(s) && is_selected(s, "EliverLara") && !s.contains(marker)
        })?;
        assert_eq!(
            restored.with_styles().to_string(),
            info_before.with_styles().to_string()
        );
    }
    terminal.send_str("pxm")?;
    terminal.send(Key::Tab)?;
    terminal.wait_frame(|s| current_focused(s) && credits(s, "Current", "Andromeda"))?;
    terminal.send(Key::BackTab)?;
    terminal.wait_frame(|s| info_focused(s) && is_selected(s, "EliverLara"))?;
    terminal.send(Key::Char('t'))?;
    terminal.wait_frame(|s| credits(s, "Pin", "3024 Day") && is_selected(s, "0x3024"))?;
    terminal.send(Key::Char('t'))?;
    terminal.wait_frame(|s| credits(s, "Current", "Andromeda") && is_selected(s, "memije"))?;
    terminal.send(Key::F(1))?;
    terminal.wait_frame(|s| s.contains("Help · Theme info"))?;
    terminal.send(Key::Char('t'))?;
    terminal.send(Key::Enter)?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| credits(s, "Current", "Andromeda"))?;
    terminal.send(Key::Esc)?;
    let restored = terminal.wait_frame(|s| !panel(s) && s.row_text(0).contains(mode))?;
    assert_eq!(
        restored.with_styles().to_string(),
        before.with_styles().to_string()
    );
    assert_eq!(fs::read(&workspace).unwrap(), saved);
    assert!(!marks.exists());
    quit(&mut terminal)
}

#[rstest]
#[case(60, 22, None)]
#[case(120, 36, None)]
#[case(60, 22, Some(false))]
#[case(120, 36, Some(false))]
#[case(120, 36, Some(true))]
fn detail_panels_restore_each_others_selection_and_focus(
    #[case] width: u16,
    #[case] height: u16,
    #[case] comparison_vertical: Option<bool>,
    #[values(false, true)] info_first: bool,
    #[values(false, true)] current_focus: bool,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(width, height)?;
    support::wait_for_explore(&mut terminal)?;
    terminal.send(Key::Char('p'))?;
    apply_query(&mut terminal, "Andromeda", 1)?;
    if let Some(vertical) = comparison_vertical {
        terminal.send(Key::Char('c'))?;
        if vertical {
            terminal.send(Key::Char('v'))?;
        }
    }
    let (parent, child) = if info_first { ('i', 'd') } else { ('d', 'i') };
    terminal.send(Key::Char(parent))?;
    if info_first {
        terminal.send(Key::Char('l'))?;
    }
    terminal.send(Key::Char('j'))?;
    if current_focus {
        terminal.send(Key::Tab)?;
    }
    let parent_ready = |s: &Screen| {
        if info_first {
            credits(s, "Current", "Andromeda")
                && info_focused(s) != current_focus
                && (current_focus || is_selected(s, "EliverLara"))
        } else {
            s.contains("Color details")
                && !panel(s)
                && is_active_label(s, "Color details") != current_focus
                && s.contains("Stored value")
        }
    };
    let before = terminal.wait_frame(parent_ready)?;
    for close in [Key::Esc, Key::Char(child), Key::Char(parent)] {
        terminal.send(Key::Char(child))?;
        terminal.wait_frame(|s| {
            if info_first {
                s.contains("Color details") && is_active_label(s, "Color details") && !panel(s)
            } else {
                info_focused(s) && s.contains("Theme info · Overview")
            }
        })?;
        terminal.send(Key::Char('f'))?;
        terminal.wait_frame(|s| s.contains("Filters · Draft"))?;
        terminal.send(Key::Esc)?;
        terminal.wait_frame(|s| {
            !s.contains("Filters · Draft")
                && if info_first {
                    is_active_label(s, "Color details")
                } else {
                    info_focused(s)
                }
        })?;
        terminal.send(close)?;
        let restored = terminal.wait_frame(parent_ready)?;
        assert_eq!(
            restored.with_styles().to_string(),
            before.with_styles().to_string()
        );
    }
    terminal.send(Key::Char(parent))?;
    terminal.wait_frame(|s| !panel(s) && !s.contains("Color details"))?;
    terminal.send(Key::Char(parent))?;
    terminal.wait_frame(|s| {
        if info_first {
            info_focused(s) && s.contains("Theme info · Overview")
        } else {
            is_active_label(s, "Color details") && s.contains("Stored value")
        }
    })?;
    quit(&mut terminal)
}

#[rstest]
#[case("Explore", "pj")]
#[case("Compare", "pjcz")]
#[case("Compare", "pjczv")]
fn info_focus_allows_toggling_the_theme_list_and_controls_its_highlight(
    #[case] mode: &str,
    #[case] keys: &str,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(120, 52)?;
    support::wait_for_explore(&mut terminal)?;
    terminal.send_str(keys)?;
    let theme = "3024 Night";
    let before = wait_for(&mut terminal, |s| {
        s.row_text(0).contains(mode) && is_selected(s, theme)
    })?;
    let (row, column) = before
        .find_all(theme)
        .into_iter()
        .find(|&(row, column)| before.cell(row, column).unwrap().style().reverse)
        .unwrap();
    let row_is_reversed = |s: &Screen| {
        assert!(s.row_text(4).contains("Themes"));
        assert!(s.row_text(row).contains(theme));
        s.cell(row, column).unwrap().style().reverse
    };

    terminal.send(Key::Char('i'))?;
    let info = wait_for(&mut terminal, info_focused)?;
    assert!(!row_is_reversed(&info));
    assert!(!is_active_label(&info, "Themes"));

    terminal.send(Key::F(1))?;
    let help = wait_for(&mut terminal, |s| s.contains("Help · Theme info"))?;
    let (row, _) = help.find("Show or hide the theme list").unwrap();
    assert_eq!(help.row_text(row).split_whitespace().next(), Some("z"));
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| panel(s) && info_focused(s))?;

    terminal.send(Key::Char('z'))?;
    wait_for(&mut terminal, |s| {
        info_focused(s) && !s.row_text(4).contains("Themes")
    })?;
    terminal.send(Key::Char('z'))?;
    let reopened = wait_for(&mut terminal, |s| {
        info_focused(s) && s.row_text(4).contains("Themes")
    })?;
    assert!(!row_is_reversed(&reopened));

    terminal.send(Key::Tab)?;
    let current = wait_for(&mut terminal, current_focused)?;
    assert!(row_is_reversed(&current));
    assert!(is_active_label(&current, "Themes"));

    terminal.send(Key::Char('z'))?;
    wait_for(&mut terminal, |s| {
        current_focused(s) && !theme_list_visible(s)
    })?;
    terminal.send(Key::Char('z'))?;
    let current = wait_for(&mut terminal, |s| {
        current_focused(s) && theme_list_visible(s)
    })?;
    assert!(row_is_reversed(&current));

    terminal.send(Key::Tab)?;
    let info = wait_for(&mut terminal, info_focused)?;
    assert!(!row_is_reversed(&info));

    let narrow = resize_and_wait(&mut terminal, 60, 22, |s| {
        info_focused(s) && s.row_text(4).contains("Theme info ·")
    })?;
    assert!(!theme_list_visible(&narrow));
    terminal.send(Key::Char('z'))?;
    terminal.send(Key::Tab)?;
    wait_for(&mut terminal, |s| {
        current_focused(s) && s.row_text(4).contains("Theme info ·")
    })?;
    resize_and_wait(&mut terminal, 120, 52, |s| {
        current_focused(s) && !s.row_text(4).contains("Theme info ·") && !theme_list_visible(s)
    })?;
    terminal.send(Key::Tab)?;
    wait_for(&mut terminal, |s| info_focused(s) && !theme_list_visible(s))?;
    terminal.send(Key::Char('z'))?;
    let reopened = wait_for(&mut terminal, |s| info_focused(s) && theme_list_visible(s))?;
    assert!(!row_is_reversed(&reopened));

    terminal.send(Key::Esc)?;
    let closed = wait_for(&mut terminal, |s| !panel(s))?;
    assert!(row_is_reversed(&closed));
    quit(&mut terminal)
}

#[test]
fn minimum_info_fits_overview_and_keeps_credit_selection_across_sections_and_resize()
-> termlens::Result<()> {
    let mut terminal = support::isolated(60, 22)?;
    support::wait_for_explore(&mut terminal)?;
    apply_query(&mut terminal, "Firewatch", 1)?;
    terminal.send(Key::Char('i'))?;
    let overview = terminal.wait_frame(|s| panel(s) && s.contains("Appearance"))?;
    insta::assert_snapshot!("info_overview_minimum", overview.with_styles());
    assert!(overview.contains("Ratio"));
    assert!(!overview.contains("┃"));
    terminal.send(Key::Char('l'))?;
    let top = terminal
        .wait_frame(|s| credits(s, "Current", "Firewatch") && is_selected(s, "Sebastian Szturo"))?;
    insta::assert_snapshot!("info_minimum", top.with_styles());
    terminal.send(Key::Char('G'))?;
    terminal.wait_frame(|s| {
        is_selected(s, "Firewatch game") && s.contains("http://www.firewatchgame.com/")
    })?;
    terminal.send(Key::Char('h'))?;
    terminal.wait_frame(|s| panel(s) && s.contains("Ratio"))?;
    terminal.send(Key::Char('l'))?;
    terminal.wait_frame(|s| is_selected(s, "Firewatch game"))?;
    resize_and_wait(&mut terminal, 59, 22, |s| s.contains("Current:  59x22"))?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| s.contains("Current:  59x22"))?;
    terminal.send(Key::Tab)?;
    wait_for(&mut terminal, |s| s.contains("Current:  59x22"))?;
    resize_and_wait(&mut terminal, 120, 36, |s| {
        credits(s, "Current", "Firewatch")
            && is_selected(s, "Firewatch game")
            && !s.row_text(4).contains("Theme info ·")
            && footer_contains(s, "F1 Help")
    })?;
    resize_and_wait(&mut terminal, 60, 22, |s| {
        credits(s, "Current", "Firewatch") && is_selected(s, "Firewatch game")
    })?;
    terminal.send(Key::Char('g'))?;
    terminal.wait_frame(|s| is_selected(s, "Sebastian Szturo"))?;
    terminal.send(Key::Char('i'))?;
    terminal.wait_frame(|s| !panel(s))?;
    terminal.send(Key::Char('i'))?;
    terminal
        .wait_frame(|s| panel(s) && s.contains("Appearance") && !s.contains("Sebastian Szturo"))?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| !panel(s))?;
    quit(&mut terminal)
}

#[test]
fn info_handles_missing_credits_single_objects_and_empty_results() -> termlens::Result<()> {
    let mut terminal = support::isolated(80, 24)?;
    support::wait_for_explore(&mut terminal)?;
    apply_query(&mut terminal, "Aardvark Blue", 1)?;
    terminal.send_str("il")?;
    let missing = terminal.wait_frame(|s| {
        credits(s, "Current", "Aardvark Blue") && s.contains("No credits provided")
    })?;
    assert!(!missing.contains("Enter Open"));
    terminal.send(Key::Enter)?;
    terminal.send(Key::Char('t'))?;
    terminal.send(Key::Char('G'))?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !panel(s))?;
    apply_query(&mut terminal, "Tomorrow Night Eighties", 1)?;
    terminal.send_str("il")?;
    terminal.wait_frame(|s| {
        credits(s, "Current", "Tomorrow Night Eighties") && s.contains("tinted-theming")
    })?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !panel(s))?;
    terminal.send(Key::Char('p'))?;
    apply_query(&mut terminal, "NO_SUCH_THEME", 0)?;
    terminal.send_str("il")?;
    terminal.wait_frame(|s| credits(s, "Pin", "Tomorrow Night Eighties"))?;
    terminal.send(Key::Char('t'))?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| !panel(s))?;
    apply_query(&mut terminal, "Tomorrow Night Eighties", 1)?;
    terminal.send(Key::Char('p'))?;
    apply_query(&mut terminal, "NO_SUCH_THEME", 0)?;
    terminal.send(Key::Char('i'))?;
    terminal.wait_frame(|s| footer_contains(s, "No theme to inspect"))?;
    apply_query(&mut terminal, "Tomorrow Night Eighties", 1)?;
    terminal.send(Key::Char('i'))?;
    wait_for(&mut terminal, panel)?;
    let empty = apply_query(&mut terminal, "NO_SUCH_THEME", 0)?;
    assert!(panel(&empty));
    terminal.send(Key::Char('d'))?;
    let unchanged = wait_for(&mut terminal, |s| {
        panel(s) && footer_contains(s, "No theme to inspect")
    })?;
    assert!(!unchanged.contains("Color details"));
    quit(&mut terminal)
}

#[rstest]
#[case(60, 22)]
#[case(120, 36)]
fn comparison_overview_tracks_current_theme_and_pin_and_copy_uses_t_without_moving_panel_focus(
    #[case] width: u16,
    #[case] height: u16,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(width, height)?;
    support::wait_for_explore(&mut terminal)?;
    apply_query(&mut terminal, "3024", 2)?;
    terminal.send_str("pjci")?;
    let overview = terminal.wait_frame(|s| {
        panel(s) && s.contains("Pin") && s.contains("Current") && s.contains("Very light")
    })?;
    insta::assert_snapshot!(
        if width == 60 {
            "info_comparison_minimum"
        } else {
            "info_comparison"
        },
        overview.with_styles()
    );
    terminal.send(Key::Char('t'))?;
    terminal.send(Key::Char('G'))?;
    terminal.wait_frame(|s| panel(s) && s.contains("Ratio"))?;
    terminal.send(Key::Tab)?;
    terminal.send(Key::Char('k'))?;
    terminal
        .wait_frame(|s| current_focused(s) && s.contains("Ratio") && !s.contains("3024 Night"))?;
    terminal.send(Key::Char('j'))?;
    terminal.send(Key::Char('y'))?;
    terminal.wait_frame(|s| s.contains("Copy theme") && s.contains("Target: Current"))?;
    terminal.send(Key::Tab)?;
    terminal.send(Key::Char('t'))?;
    terminal.wait_frame(|s| s.contains("Target: Pin"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_frame(|s| current_focused(s) && s.contains("Ratio"))?;
    terminal.send(Key::Char('p'))?;
    terminal.wait_frame(|s| panel(s) && s.contains("Pin") && !s.contains("3024 Day"))?;
    terminal.send(Key::Tab)?;
    terminal.send(Key::Char('l'))?;
    terminal.wait_frame(|s| credits(s, "Current", "3024 Night"))?;
    terminal.send(Key::Char('t'))?;
    wait_for(&mut terminal, |s| credits(s, "Pin", "3024 Night"))?;
    terminal.send(Key::Esc)?;
    wait_for(&mut terminal, |s| {
        !panel(s) && s.row_text(0).contains("Compare")
    })?;
    apply_query(&mut terminal, "NO_SUCH_THEME", 0)?;
    terminal.send(Key::Char('i'))?;
    wait_for(&mut terminal, |s| {
        panel(s)
            && s.find("Theme info · Overview").is_some_and(|(row, _)| {
                let heading = s.row_text(row + 2);
                heading.trim_start().starts_with(if width == 60 {
                    "Pin "
                } else {
                    "Pin: 3024 Night"
                }) && heading.trim_end().ends_with("Current")
                    && if width == 60 {
                        s.row_text(row + 3).contains("No theme select")
                            && s.row_text(row + 4).trim_start().starts_with("ed")
                    } else {
                        s.row_text(row + 3).contains("No theme selected")
                    }
            })
    })?;
    terminal.send(Key::Char('G'))?;
    wait_for(&mut terminal, |s| panel(s) && s.contains("Ratio"))?;
    quit(&mut terminal)
}
