use ratatui::{
    Terminal,
    backend::TestBackend,
    crossterm::event::{KeyCode, KeyModifiers},
    style::Color,
};
use rstest::rstest;

use super::*;
use crate::{
    keybindings::KeyChord,
    test_support::{comparison_in_context, custom_navigation_config, render},
};

#[test]
fn browser_opening_uses_the_complete_selected_url_and_preserves_the_panel_on_failure_and_retry() {
    let mut app = App::default();
    let config = Config::default();
    app.update(Action::EditQuery, 14);
    app.update(Action::ClearInput, 14);
    crate::test_support::insert_text(&mut app, "MaterialOcean");
    app.update(Action::Confirm, 14);
    app.workspace_action = None;
    app.update(Action::OpenThemeInfo, 14);
    app.update(Action::ValueNext, 14);
    let mut terminal = Terminal::new(TestBackend::new(60, 22)).unwrap();
    render(&mut terminal, &mut app, &config);
    assert_eq!(app.theme_info.as_ref().unwrap().selected, Some(0));
    let offset = app.theme_info.as_ref().unwrap().offset;
    let before = terminal.backend().buffer().clone();
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        key.kind = kind;
        handle_key(&mut app, key, Size::new(60, 22), &config);
        assert!(app.browser_action.is_none());
    }
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        Size::new(60, 22),
        &config,
    );
    process_browser(&mut app, |url| {
        assert_eq!(
            url,
            "https://github.com/kaicataldo/material.vim/blob/master/terminal-colors/iterm2/material-ocean.itermcolors"
        );
        Err(io::Error::other(format!(
            "{} CAUSE_END",
            "launch failed ".repeat(20)
        )))
    });
    render(&mut terminal, &mut app, &config);
    let failed = terminal.backend().buffer();
    let footer = layout::screen(failed.area).unwrap().footer;
    assert_eq!(failed[(footer.x, footer.y)].fg, Color::Red);
    let footer_text: String = (footer.x..footer.right())
        .map(|x| failed[(x, footer.y)].symbol())
        .collect();
    assert!(footer_text.starts_with("Error: Could not open browser: launch failed"));
    assert_eq!(app.theme_info.as_ref().unwrap().selected, Some(0));
    assert_eq!(app.theme_info.as_ref().unwrap().offset, offset);
    for y in 0..footer.y {
        for x in 0..60 {
            assert_eq!(failed[(x, y)], before[(x, y)]);
        }
    }

    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        Size::new(60, 22),
        &config,
    );
    process_browser(&mut app, |url| {
        assert!(url.ends_with("material-ocean.itermcolors"));
        Ok(())
    });
    render(&mut terminal, &mut app, &config);
    assert_eq!(terminal.backend().buffer(), &before);
    assert_eq!(app.context(), InputContext::ThemeInfo);
    assert!(app.browser_action.is_none());
    process_browser(&mut app, |_| {
        panic!("the request must be consumed only once")
    });
    assert!(app.workspace_action.is_none());
    assert!(app.storage_action.is_none());
}

#[rstest]
#[case(false, 36, 15)]
#[case(true, 48, 27)]
fn comparison_list_pages_follow_the_visible_height_without_changing_color_focus(
    #[case] vertical: bool,
    #[case] height: u16,
    #[case] paged_selection: usize,
) {
    let mut app = comparison_in_context(InputContext::Browse);
    let config = Config::default();
    if vertical {
        app.update(Action::ToggleLayout, 40);
    }
    app.workspace_action = None;
    let size = Size::new(120, height);
    for code in [KeyCode::Char('d'), KeyCode::Char('z')] {
        handle_key(
            &mut app,
            KeyEvent::new(code, KeyModifiers::NONE),
            size,
            &config,
        );
    }
    assert!(app.theme_list_visible());
    assert_eq!(app.context(), InputContext::Colors);
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE);
        key.kind = kind;
        handle_key(&mut app, key, size, &config);
        assert!(app.theme_list_visible());
    }
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
        size,
        &config,
    );
    assert_eq!(app.selected(), Some(1));
    assert_eq!(app.color_details.list.selected(), Some(7));
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
        size,
        &config,
    );
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
        size,
        &config,
    );
    assert_eq!(app.selected(), Some(paged_selection));
    assert_eq!(app.color_details.list.selected(), Some(7));
    let layout = layout::screen(Rect::new(0, 0, 120, height)).unwrap();
    let colors = layout.colors(&app).unwrap();
    assert_eq!(colors.area.x, layout.main.x);
    assert_eq!(colors.area.width, layout.main.width);
    assert!(colors.themes.unwrap().bottom() < colors.area.y);
    assert!(app.workspace_action.is_none());
}

#[test]
fn colors_use_their_actual_page_height_and_remapped_focus_without_moving_the_current_theme() {
    let mut app = comparison_in_context(InputContext::Browse);
    for _ in 0..12 {
        app.update(Action::Next, 14);
    }
    let mut config = Config::default();
    for (key, action) in &mut config.keybindings.bindings {
        let code = match action {
            Action::ToggleColors => KeyCode::F(6),
            Action::FieldNext => KeyCode::F(7),
            _ => continue,
        };
        *key = KeyChord::new(code, KeyModifiers::NONE);
    }
    let before = app.theme_list;
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE),
        Size::new(80, 24),
        &config,
    );
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
        Size::new(80, 24),
        &config,
    );
    assert_eq!(app.color_details.list.selected(), Some(4));
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
        Size::new(60, 22),
        &config,
    );
    assert_eq!(app.color_details.list.selected(), Some(13));
    assert_eq!(app.theme_list, before);
    for code in [KeyCode::F(7), KeyCode::Char('j'), KeyCode::Esc] {
        handle_key(
            &mut app,
            KeyEvent::new(code, KeyModifiers::NONE),
            Size::new(59, 22),
            &config,
        );
    }
    assert_eq!(app.context(), InputContext::Colors);
    assert_eq!(app.color_details.list.selected(), Some(13));
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::F(7), KeyModifiers::NONE),
        Size::new(80, 24),
        &config,
    );
    assert_eq!(app.theme_list, before);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
        Size::new(80, 24),
        &config,
    );
    assert_eq!(app.selected(), Some(29));
    assert_eq!(app.color_details.list.selected(), Some(13));
    let help = ui::help::detail_lines(&app, &config, app.context(), 76)
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(help.contains("F6"));
    assert!(help.contains("F7  S-Tab"));
}

#[test]
fn copy_remaps_open_target_and_retry_without_repeating_the_operation() {
    use crate::app::theme_target::ThemeTarget;

    let mut app = App::default();
    let mut config = Config::default();
    for (key, action) in &mut config.keybindings.bindings {
        if *action == Action::OpenCopy {
            *key = KeyChord::new(KeyCode::F(6), KeyModifiers::NONE);
        }
    }
    for (key, action) in &mut config.keybindings.menu {
        let code = match action {
            Action::Confirm => KeyCode::F(7),
            Action::SwitchThemeTarget => KeyCode::F(8),
            _ => continue,
        };
        *key = KeyChord::new(code, KeyModifiers::NONE);
    }
    type_text(&mut app, "pjy", &config);
    assert!(app.copy_selection.is_none());
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE);
        key.kind = kind;
        handle_key(&mut app, key, Size::new(80, 24), &config);
        assert!(app.copy_selection.is_none());
    }
    press(&mut app, KeyCode::F(6), KeyModifiers::NONE, &config);
    for code in [
        KeyCode::F(8),
        KeyCode::F(7),
        KeyCode::Char('j'),
        KeyCode::Esc,
    ] {
        handle_key(
            &mut app,
            KeyEvent::new(code, KeyModifiers::NONE),
            Size::new(59, 22),
            &config,
        );
        assert_eq!(
            app.copy_selection.as_ref().unwrap().target,
            ThemeTarget::Current
        );
        assert_eq!(
            app.copy_selection.as_ref().unwrap().format,
            crate::copy::CopyFormat::Name
        );
        assert!(app.copy_action.is_none());
    }
    for code in [KeyCode::F(8), KeyCode::F(7), KeyCode::Esc] {
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            let mut key = KeyEvent::new(code, KeyModifiers::NONE);
            key.kind = kind;
            handle_key(&mut app, key, Size::new(80, 24), &config);
            assert_eq!(
                app.copy_selection.as_ref().unwrap().target,
                ThemeTarget::Current
            );
            assert!(app.copy_action.is_none());
        }
    }
    press(&mut app, KeyCode::F(8), KeyModifiers::NONE, &config);
    press(&mut app, KeyCode::F(7), KeyModifiers::NONE, &config);
    let request = app.copy_action.take().unwrap();
    assert_eq!(request.text, "3024 Day");
    app.copy_finished(request, Err("access denied".into()));
    let footer = ui::help::modal_footer(&app, 78, &config.keybindings, config.ui_theme).to_string();
    assert!(footer.contains("F7 Copy"));
    assert!(footer.contains("F8 Target"));
    assert!(!footer.contains("Enter"));
    let details = ui::help::detail_lines(&app, &config, app.context(), 76);
    assert!(details.iter().any(|line| {
        line.to_string().starts_with("F8 ")
            && line.to_string().contains("Switch between Current and Pin")
    }));
    press(&mut app, KeyCode::F(7), KeyModifiers::NONE, &config);
    let request = app.copy_action.take().unwrap();
    assert_eq!(request.text, "3024 Day");
    app.copy_finished(request, Ok(()));
    assert!(app.copy_selection.is_none());
    assert!(
        matches!(app.notice, Some(crate::app::Notice::Success(ref text)) if text == "Copied Name: 3024 Day")
    );
    assert_eq!(app.theme().unwrap().name, "3024 Night");
    assert!(app.workspace_action.is_none());
    assert!(app.storage_action.is_none());
}

#[rstest]
#[case(59, 22)]
#[case(60, 21)]
fn navigation_while_small_keeps_the_preview_on_recovery(#[case] width: u16, #[case] height: u16) {
    let mut app = App::default();
    let config = Config::default();
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        Size::new(80, 24),
        &config,
    );
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('['), KeyModifiers::NONE),
        Size::new(width, height),
        &config,
    );
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
        Size::new(width, height),
        &config,
    );
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    render(&mut terminal, &mut app, &config);
    let title: String = (26..80)
        .map(|col| terminal.backend().buffer()[(col, 4)].symbol())
        .collect();
    assert!(title.contains("3024 Night"));
    assert!(
        (0..80)
            .map(|x| terminal.backend().buffer()[(x, 0)].symbol())
            .collect::<String>()
            .contains("Preview  Palette")
    );
    assert!(app.selected() == Some(1));
}

#[test]
fn footer_notice_survives_redraw_and_release_then_clears_on_an_unbound_key() {
    let mut app = App::default();
    let config = Config::default();
    press(&mut app, KeyCode::Char('c'), KeyModifiers::NONE, &config);
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let footer: String = (1..99).map(|x| buffer[(x, 29)].symbol()).collect();
    assert!(footer.contains("Pin a theme first (p)"));
    assert!(!footer.contains("Move"));
    let mut release = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    release.kind = KeyEventKind::Release;
    handle_key(&mut app, release, Size::new(100, 30), &config);
    assert!(app.notice.is_some());
    press(&mut app, KeyCode::Char('u'), KeyModifiers::NONE, &config);
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let footer: String = (1..99).map(|x| buffer[(x, 29)].symbol()).collect();
    assert!(footer.contains("Move"));
    assert!(!footer.contains("Pin a theme first"));
}

#[test]
fn custom_bindings_replace_defaults_in_runtime_dispatch() {
    let mut app = App::default();
    let config = custom_navigation_config();
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
        Size::new(80, 24),
        &config,
    );
    let mut repeated = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
    repeated.kind = KeyEventKind::Repeat;
    handle_key(&mut app, repeated, Size::new(80, 24), &config);
    let mut released = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    released.kind = KeyEventKind::Release;
    handle_key(&mut app, released, Size::new(80, 24), &config);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        Size::new(80, 24),
        &config,
    );
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
        Size::new(80, 24),
        &config,
    );
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char(']'), KeyModifiers::NONE),
        Size::new(80, 24),
        &config,
    );

    assert_eq!(app.theme().unwrap().name, "Aardvark Blue");
    assert_eq!(
        app.preview_sample,
        crate::preview::Sample::BuiltIn(crate::preview::Preset::Shell)
    );
    assert_eq!(app.selected(), Some(2));

    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
        Size::new(80, 24),
        &config,
    );
    assert!(matches!(app.notice, Some(crate::app::Notice::PinRequired)));

    for key in ['a', 'p', 'c', 'v', 's', 't'] {
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE),
            Size::new(80, 24),
            &config,
        );
    }
    assert_eq!(app.theme().unwrap().name, "Aardvark Blue");
    assert_eq!(app.pinned_theme().unwrap().name, "Aardvark Blue");
    assert_eq!(app.mode, crate::app::Mode::Compare);
    assert_eq!(app.compare_direction, ratatui::layout::Direction::Vertical);
}

#[rstest]
#[case(59, 22, false)]
#[case(60, 21, true)]
fn comparison_rejects_changes_while_too_small(
    #[case] width: u16,
    #[case] height: u16,
    #[case] vertical: bool,
) {
    let mut app = App::default();
    let config = Config::default();
    let mut keys = vec!['p', 'j', 'c'];
    if vertical {
        keys.push('v');
    }
    for key in keys {
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE),
            Size::new(80, 24),
            &config,
        );
    }
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    render(&mut terminal, &mut app, &config);
    let before = terminal.backend().buffer().clone();
    for key in ['p', 'c', 'v', 'j', '[', ' ', 'x', 'r'] {
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE),
            Size::new(width, height),
            &config,
        );
    }
    render(&mut terminal, &mut app, &config);
    assert_eq!(terminal.backend().buffer(), &before);
    assert!(app.storage_action.is_none());
}

fn press(app: &mut App, code: KeyCode, modifiers: KeyModifiers, config: &Config) {
    handle_key(
        app,
        KeyEvent::new(code, modifiers),
        Size::new(80, 24),
        config,
    );
}

fn type_text(app: &mut App, text: &str, config: &Config) {
    for (index, character) in text.chars().enumerate() {
        press(app, KeyCode::Char(character), KeyModifiers::NONE, config);
        if index == 0 && character == '/' {
            press(app, KeyCode::Char('u'), KeyModifiers::CONTROL, config);
        }
    }
}

#[rstest]
#[case(' ', crate::marks::Mark::Favorite)]
#[case('x', crate::marks::Mark::Ignored)]
fn mark_toggles_ignore_repeat_and_release_and_use_remapped_keys(
    #[case] key: char,
    #[case] mark: crate::marks::Mark,
) {
    let mut app = App::default();
    let mut config = Config::default();
    let action = if key == ' ' {
        Action::ToggleFavorite
    } else {
        Action::ToggleIgnore
    };
    for (chord, bound) in &mut config.keybindings.bindings {
        if *bound == action {
            *chord = KeyChord::new(KeyCode::F(6), KeyModifiers::NONE);
        }
    }
    press(&mut app, KeyCode::Char(key), KeyModifiers::NONE, &config);
    assert!(app.storage_action.is_none());
    press(&mut app, KeyCode::F(6), KeyModifiers::NONE, &config);
    let Some(change) = app.storage_action.take() else {
        panic!("expected a save request")
    };
    assert_eq!(change.mark, Some(mark));
    app.mark_saved(change);
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut event = KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE);
        event.kind = kind;
        handle_key(&mut app, event, Size::new(80, 24), &config);
        assert!(app.storage_action.is_none());
    }
    assert_eq!(app.marks.len(), 1);
}

#[rstest]
#[case(Action::RenameFilter, 'e')]
#[case(Action::DeleteFilter, 'D')]
fn saved_dialogs_use_remapped_keys_and_ignore_repeated_confirm_and_cancel(
    #[case] action: Action,
    #[case] original: char,
) {
    use crate::workspace::{WorkspaceAction, WorkspaceChange};

    let mut app = App::default();
    app.workspace = app
        .workspace
        .changed(&WorkspaceChange::New {
            name: "Dark".into(),
            query: "type:dark".into(),
        })
        .unwrap();
    let mut config = Config::default();
    for (chord, bound) in &mut config.keybindings.saved {
        if *bound == action {
            *chord = KeyChord::new(KeyCode::F(6), KeyModifiers::NONE);
        }
        if *bound == Action::Confirm {
            *chord = KeyChord::new(KeyCode::F(2), KeyModifiers::NONE);
        }
        if *bound == Action::Cancel {
            *chord = KeyChord::new(KeyCode::F(3), KeyModifiers::NONE);
        }
    }
    for (chord, bound) in &mut config.keybindings.editing {
        if *bound == Action::Confirm {
            *chord = KeyChord::new(KeyCode::F(2), KeyModifiers::NONE);
        }
        if *bound == Action::Cancel {
            *chord = KeyChord::new(KeyCode::F(3), KeyModifiers::NONE);
        }
    }
    type_text(&mut app, "s", &config);
    press(&mut app, KeyCode::Char('j'), KeyModifiers::NONE, &config);
    press(
        &mut app,
        KeyCode::Char(original),
        KeyModifiers::NONE,
        &config,
    );
    assert!(app.dialog.is_none());
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE);
        key.kind = kind;
        handle_key(&mut app, key, Size::new(80, 24), &config);
        assert!(app.dialog.is_none());
    }
    press(&mut app, KeyCode::F(6), KeyModifiers::NONE, &config);
    if action == Action::RenameFilter {
        assert_eq!(app.dialog.as_ref().unwrap().input.cursor(), 4);
        type_text(&mut app, "q", &config);
    } else {
        type_text(&mut app, "nexq /", &config);
    }
    assert!(!app.quit);
    assert!(app.storage_action.is_none());
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let footer: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
    assert!(footer.contains(if action == Action::RenameFilter {
        "F2 Rename"
    } else {
        "F2 Delete"
    }));
    assert!(footer.contains("F3 Cancel"));
    press(&mut app, KeyCode::Enter, KeyModifiers::NONE, &config);
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        for code in [KeyCode::F(2), KeyCode::F(3)] {
            let mut key = KeyEvent::new(code, KeyModifiers::NONE);
            key.kind = kind;
            handle_key(&mut app, key, Size::new(80, 24), &config);
            assert!(app.dialog.is_some());
            assert!(app.workspace_action.is_none());
        }
    }
    press(&mut app, KeyCode::F(2), KeyModifiers::NONE, &config);
    let Some(WorkspaceAction::Filter(change)) = app.workspace_action.take() else {
        panic!("expected one explicit save")
    };
    let next = app.workspace_snapshot(Some(&change)).unwrap();
    app.workspace_saved(next, Some(change));
    assert!(app.dialog.is_none());
    assert!(app.saved.is_some());
    assert!(!app.quit);
    if action == Action::RenameFilter {
        assert_eq!(app.workspace.saved_filters[0].name, "Darkq");
    } else {
        assert!(app.workspace.saved_filters.is_empty());
    }
    press(&mut app, KeyCode::F(3), KeyModifiers::NONE, &config);
    assert!(app.saved.is_none());
    assert!(app.filter.is_default());
}

#[rstest]
#[case(false)]
#[case(true)]
fn editor_input_is_preserved_while_too_small(#[case] filters: bool) {
    let mut app = App::default();
    let config = Config::default();
    type_text(&mut app, if filters { "f" } else { "/" }, &config);
    if filters {
        press(&mut app, KeyCode::BackTab, KeyModifiers::NONE, &config);
    }
    type_text(&mut app, "\"catpp", &config);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    render(&mut terminal, &mut app, &config);
    let before = terminal.backend().buffer().clone();
    for code in [
        KeyCode::Esc,
        KeyCode::Char('R'),
        KeyCode::Char('j'),
        KeyCode::Tab,
        KeyCode::Right,
        KeyCode::Enter,
    ] {
        handle_key(
            &mut app,
            KeyEvent::new(code, KeyModifiers::NONE),
            Size::new(59, 22),
            &config,
        );
    }
    render(&mut terminal, &mut app, &config);
    assert_eq!(terminal.backend().buffer(), &before);
    assert!(!app.quit);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        Size::new(59, 22),
        &config,
    );
    assert!(app.quit);
}

#[rstest]
#[case(KeyCode::BackTab, KeyModifiers::NONE)]
#[case(KeyCode::BackTab, KeyModifiers::SHIFT)]
#[case(KeyCode::Tab, KeyModifiers::SHIFT)]
fn custom_editing_bindings_drive_the_form_and_its_help(
    #[case] previous: KeyCode,
    #[case] modifiers: KeyModifiers,
) {
    let mut app = App::default();
    let mut config = Config::default();
    config.keybindings.editing.retain(|(_, action)| {
        !matches!(
            action,
            Action::Confirm | Action::Cancel | Action::FieldNext | Action::ClearFilters
        )
    });
    for (number, action) in [
        (2, Action::Confirm),
        (3, Action::Cancel),
        (4, Action::FieldNext),
        (5, Action::ClearFilters),
    ] {
        config.keybindings.editing.push((
            KeyChord::new(KeyCode::F(number), KeyModifiers::NONE),
            action,
        ));
    }
    type_text(&mut app, "/catpp", &config);
    press(&mut app, KeyCode::Enter, KeyModifiers::NONE, &config);
    assert!(app.query_editor().is_some());
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    render(&mut terminal, &mut app, &config);
    let text: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(text.contains("F2 Apply  F3 Cancel"));
    press(&mut app, KeyCode::F(2), KeyModifiers::NONE, &config);
    type_text(&mut app, "f", &config);
    press(&mut app, KeyCode::F(5), KeyModifiers::NONE, &config);
    assert_eq!(app.themes().len(), 8);
    press(&mut app, previous, modifiers, &config);
    assert_eq!(app.context(), InputContext::FilterName);
    press(&mut app, KeyCode::F(4), KeyModifiers::NONE, &config);
    assert_eq!(app.context(), InputContext::FilterChoice);
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let footer: String = (0..80).map(|x| buffer[(x, 23)].symbol()).collect();
    assert!(footer.contains("F2 Apply  F3 Cancel"));
    assert!(!footer.contains("F5 Reset"));
    assert!(footer.contains("F4/S-Tab Field"));
    let details = ui::help::detail_lines(&app, &config, app.context(), 76);
    assert!(
        details
            .iter()
            .any(|line| line.to_string().starts_with("F4 ")
                && line.to_string().contains("Edit the next filter field"))
    );
    let body: String = (3..22)
        .flat_map(|y| (26..79).map(move |x| buffer[(x, y)].symbol()))
        .collect();
    assert!(!body.contains("Apply"));
    assert!(!body.contains("Cancel"));
    assert!(!body.contains("Clear"));
    for _ in 0..5 {
        press(&mut app, KeyCode::F(4), KeyModifiers::NONE, &config);
        assert_eq!(app.context(), InputContext::FilterChoice);
    }
    press(&mut app, KeyCode::F(4), KeyModifiers::NONE, &config);
    assert_eq!(app.context(), InputContext::FilterName);
    press(&mut app, KeyCode::F(3), KeyModifiers::NONE, &config);
    assert_eq!(app.themes().len(), 8);
    assert_eq!(app.context(), InputContext::Browse);
}

#[rstest]
#[case(InputContext::Browse)]
#[case(InputContext::Query)]
#[case(InputContext::FilterName)]
#[case(InputContext::DialogName)]
#[case(InputContext::Preset)]
#[case(InputContext::Copy)]
#[case(InputContext::ThemeInfo)]
#[case(InputContext::FilterChoice)]
#[case(InputContext::Saved)]
#[case(InputContext::DialogConfirm)]
fn help_closes_one_layer_and_blocks_background_actions(#[case] context: InputContext) {
    let mut app = comparison_in_context(context);
    let config = Config::default();
    assert_eq!(app.context(), context);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    render(&mut terminal, &mut app, &config);
    let before = terminal.backend().buffer().clone();
    let query = app.filter.query();
    let current_name = app.theme().unwrap().name.to_owned();
    press(&mut app, KeyCode::F(1), KeyModifiers::NONE, &config);
    assert_eq!(app.context(), InputContext::Help);
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        for code in [KeyCode::Esc, KeyCode::F(1)] {
            let mut event = KeyEvent::new(code, KeyModifiers::NONE);
            event.kind = kind;
            handle_key(&mut app, event, Size::new(80, 24), &config);
            assert_eq!(app.context(), InputContext::Help);
        }
    }
    type_text(&mut app, "G", &config);
    render(&mut terminal, &mut app, &config);
    assert!(app.help.is_some());
    for code in [
        KeyCode::Esc,
        KeyCode::F(1),
        KeyCode::Char('R'),
        KeyCode::Char('P'),
    ] {
        handle_key(
            &mut app,
            KeyEvent::new(code, KeyModifiers::NONE),
            Size::new(59, 22),
            &config,
        );
        assert_eq!(app.context(), InputContext::Help);
    }
    press(&mut app, KeyCode::Esc, KeyModifiers::NONE, &config);
    assert_eq!(app.context(), context);
    assert_eq!(app.filter.query(), query);
    assert_eq!(app.theme().unwrap().name, current_name);
    assert!(app.workspace_action.is_none());
    assert!(app.storage_action.is_none());
    render(&mut terminal, &mut app, &config);
    assert_eq!(terminal.backend().buffer(), &before);
    assert!(!app.quit);
}

#[test]
fn remapped_common_actions_drive_their_menus_and_both_help_views() {
    let mut app = App::default();
    let mut config = Config::default();
    for (key, action) in &mut config.keybindings.bindings {
        let number = match action {
            Action::ToggleHelp => 2,
            Action::OpenPreset => 3,
            Action::ClearFilters => 4,
            _ => continue,
        };
        *key = KeyChord::new(KeyCode::F(number), KeyModifiers::NONE);
    }
    for (key, action) in &mut config.keybindings.menu {
        let number = match action {
            Action::Confirm => 5,
            Action::Cancel => 6,
            Action::ToggleHelp => 7,
            _ => continue,
        };
        *key = KeyChord::new(KeyCode::F(number), KeyModifiers::NONE);
    }
    type_text(&mut app, "/catpp", &config);
    press(&mut app, KeyCode::Enter, KeyModifiers::NONE, &config);
    let footer = ui::help::footer(&app, 200, &config.keybindings, config.ui_theme).to_string();
    assert!(footer.contains("F4 Reset"));
    assert!(footer.contains("F2 Help"));
    press(&mut app, KeyCode::F(2), KeyModifiers::NONE, &config);
    let context = app.help.as_ref().unwrap().context;
    let details = ui::help::detail_lines(&app, &config, context, 76)
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        details
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .contains("F3 Open the preview selector")
    );
    assert!(
        details
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .contains("F4 Reset all conditions to Initial")
    );
    assert!(!details.contains("P  Preview"));
    press(&mut app, KeyCode::F(6), KeyModifiers::NONE, &config);
    press(&mut app, KeyCode::F(3), KeyModifiers::NONE, &config);
    type_text(&mut app, "l", &config);
    assert!(app.workspace_action.is_some()); // The earlier confirmed condition only.
    app.workspace_action = None;
    let footer = ui::help::footer(&app, 78, &config.keybindings, config.ui_theme).to_string();
    assert!(
        ui::help::modal_footer(&app, 78, &config.keybindings, config.ui_theme)
            .to_string()
            .contains("F5 Use  F6 Cancel")
    );
    assert!(footer.contains("F7 Help"));
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE);
        key.kind = kind;
        handle_key(&mut app, key, Size::new(80, 24), &config);
        assert_eq!(app.context(), InputContext::Preset);
    }
    press(&mut app, KeyCode::F(5), KeyModifiers::NONE, &config);
    assert_eq!(
        app.preview_sample,
        crate::preview::Sample::BuiltIn(crate::preview::Preset::Shell)
    );
    assert!(app.workspace_action.is_some());
    app.workspace_action = None;
    press(&mut app, KeyCode::F(3), KeyModifiers::NONE, &config);
    press(&mut app, KeyCode::F(5), KeyModifiers::NONE, &config);
    assert!(app.workspace_action.is_none());
    press(&mut app, KeyCode::F(4), KeyModifiers::NONE, &config);
    assert!(app.filter.is_default());
}

#[rstest]
#[case(Action::ToggleThemeList, 'z')]
#[case(Action::ToggleCompare, 'c')]
#[case(Action::TogglePin, 'p')]
#[case(Action::ToggleLayout, 'v')]
fn view_toggles_use_remapped_press_only_keys(#[case] action: Action, #[case] original: char) {
    let mut app = App::default();
    let mut config = Config::default();
    type_text(&mut app, "pj", &config);
    if action == Action::ToggleLayout {
        type_text(&mut app, "c", &config);
    }
    let before = (
        app.mode,
        app.theme_list_visible(),
        app.pinned_theme().unwrap().name.to_owned(),
        app.compare_direction,
    );
    for (key, bound) in &mut config.keybindings.bindings {
        if *bound == action {
            *key = KeyChord::new(KeyCode::F(6), KeyModifiers::NONE);
        }
    }
    press(
        &mut app,
        KeyCode::Char(original),
        KeyModifiers::NONE,
        &config,
    );
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE);
        key.kind = kind;
        handle_key(&mut app, key, Size::new(80, 24), &config);
        assert_eq!(
            (
                app.mode,
                app.theme_list_visible(),
                app.pinned_theme().unwrap().name.to_owned(),
                app.compare_direction
            ),
            before
        );
    }
    press(&mut app, KeyCode::F(6), KeyModifiers::NONE, &config);
    assert_ne!(
        (
            app.mode,
            app.theme_list_visible(),
            app.pinned_theme().unwrap().name.to_owned(),
            app.compare_direction
        ),
        before
    );
    if action == Action::ToggleThemeList {
        let footer = ui::help::footer(&app, 400, &config.keybindings, config.ui_theme).to_string();
        assert!(footer.contains("F6 List"));
        let details = ui::help::detail_lines(&app, &config, InputContext::Browse, 76)
            .iter()
            .map(|line| line.to_string())
            .collect::<String>();
        assert!(
            details
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .contains("F6 Show or hide the theme list")
        );
        assert!(!details.contains("Back to Explore"));
        for code in [
            KeyCode::Char('z'),
            KeyCode::Char('c'),
            KeyCode::Char('p'),
            KeyCode::Char('v'),
            KeyCode::Esc,
        ] {
            handle_key(
                &mut app,
                KeyEvent::new(code, KeyModifiers::NONE),
                Size::new(59, 22),
                &config,
            );
            assert_eq!(app.mode, crate::app::Mode::Explore);
            assert!(!app.theme_list_visible());
        }
    } else if action == Action::ToggleCompare {
        let details = ui::help::detail_lines(&app, &config, InputContext::Browse, 76)
            .iter()
            .map(|line| line.to_string())
            .collect::<String>();
        assert!(
            details
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .contains("Esc F6 Back to Explore")
        );
        assert_eq!(details.matches("Back to Explore").count(), 1);
    }
}

#[rstest]
#[case(false, false, 24, 12)]
#[case(false, true, 24, 12)]
#[case(true, false, 8, 8)]
#[case(true, true, 17, 16)]
fn editor_pages_use_the_actual_results_height_and_keep_the_selection(
    #[case] filters: bool,
    #[case] vertical: bool,
    #[case] page: usize,
    #[case] resized_page: usize,
) {
    let mut app = App::default();
    let config = Config::default();
    type_text(&mut app, "pjc", &config);
    if vertical {
        app.compare_direction = ratatui::layout::Direction::Vertical;
    }
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(
            KeyCode::Char(if filters { 'f' } else { '/' }),
            KeyModifiers::NONE,
        );
        key.kind = kind;
        handle_key(&mut app, key, Size::new(120, 36), &config);
        assert!(app.editor.is_none());
    }
    type_text(&mut app, if filters { "f" } else { "/" }, &config);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
        Size::new(120, 36),
        &config,
    );
    if filters {
        assert_eq!(app.theme().unwrap().name, "3024 Night");
        assert_eq!(app.filters().unwrap().offset, page);
    } else {
        assert_eq!(app.selected(), Some(page + 1));
    }
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
        Size::new(80, 24),
        &config,
    );
    if filters {
        assert_eq!(app.theme().unwrap().name, "3024 Night");
        assert_eq!(app.filters().unwrap().offset, page + resized_page);
        let offset = app.filters().unwrap().offset;
        press(&mut app, KeyCode::F(1), KeyModifiers::NONE, &config);
        assert!(app.help.is_some());
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
            Size::new(120, 36),
            &config,
        );
        assert_eq!(app.help.as_ref().unwrap().offset, 28);
        press(&mut app, KeyCode::Esc, KeyModifiers::NONE, &config);
        assert!(app.filters().is_some());
        assert_eq!(app.filters().unwrap().offset, offset);
    } else {
        assert_eq!(app.selected(), Some(page + 1 + resized_page));
    }
    for code in [KeyCode::Enter, KeyCode::Esc] {
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            let mut key = KeyEvent::new(code, KeyModifiers::NONE);
            key.kind = kind;
            handle_key(&mut app, key, Size::new(80, 24), &config);
            assert!(app.editor.is_some());
        }
    }
    press(&mut app, KeyCode::Esc, KeyModifiers::NONE, &config);
    assert_eq!(app.theme().unwrap().name, "3024 Night");
}

#[test]
fn saved_screen_uses_remapped_press_only_delete_and_returns_directly() {
    let mut app = comparison_in_context(InputContext::Saved);
    let mut config = Config::default();
    for (key, action) in &mut config.keybindings.saved {
        if *action == Action::DeleteFilter {
            *key = KeyChord::new(KeyCode::F(6), KeyModifiers::NONE);
        }
    }
    press(&mut app, KeyCode::Char('j'), KeyModifiers::NONE, &config);
    let mode = app.mode;
    type_text(&mut app, "mdD", &config);
    assert!(app.dialog.is_none());
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE);
        key.kind = kind;
        handle_key(&mut app, key, Size::new(80, 24), &config);
        assert!(app.dialog.is_none());
    }
    press(&mut app, KeyCode::F(6), KeyModifiers::NONE, &config);
    assert_eq!(app.context(), InputContext::DialogConfirm);
    press(&mut app, KeyCode::Esc, KeyModifiers::NONE, &config);
    assert_eq!(app.context(), InputContext::Saved);
    press(&mut app, KeyCode::Esc, KeyModifiers::NONE, &config);
    assert_eq!(app.context(), InputContext::Browse);
    assert_eq!(app.mode, mode);
    assert_eq!(app.theme().unwrap().name, "3024 Night");
    assert_eq!(app.pinned_theme().unwrap().name, "3024 Day");
}
