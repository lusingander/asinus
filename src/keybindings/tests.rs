use rstest::rstest;

use super::*;

#[rstest]
#[case(InputContext::ThemeInfo, KeyCode::Char('t'), Action::SwitchThemeTarget)]
#[case(InputContext::Copy, KeyCode::Char('t'), Action::SwitchThemeTarget)]
#[case(InputContext::Colors, KeyCode::Char('t'), Action::SwitchThemeTarget)]
#[case(InputContext::ThemeInfo, KeyCode::Char('i'), Action::OpenThemeInfo)]
#[case(InputContext::ThemeInfo, KeyCode::Left, Action::ValuePrevious)]
#[case(InputContext::ThemeInfo, KeyCode::Right, Action::ValueNext)]
fn info_and_target_switches_ignore_repeat_and_release(
    #[case] context: InputContext,
    #[case] code: KeyCode,
    #[case] action: Action,
) {
    let bindings = KeyBindings::default();
    let mut key = KeyEvent::new(code, KeyModifiers::NONE);
    assert_eq!(bindings.action_for_key(key, context), Some(action));
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        key.kind = kind;
        assert_eq!(bindings.action_for_key(key, context), None);
    }
}

#[rstest]
#[case(KeyCode::Char('d'), KeyModifiers::NONE, Action::ToggleColors)]
#[case(KeyCode::Char('i'), KeyModifiers::NONE, Action::OpenThemeInfo)]
#[case(KeyCode::Char('Z'), KeyModifiers::NONE, Action::TogglePanelMaximize)]
#[case(KeyCode::Char('Z'), KeyModifiers::SHIFT, Action::TogglePanelMaximize)]
#[case(KeyCode::Char('z'), KeyModifiers::SHIFT, Action::TogglePanelMaximize)]
#[case(KeyCode::Tab, KeyModifiers::NONE, Action::FieldNext)]
#[case(KeyCode::BackTab, KeyModifiers::NONE, Action::FieldPrevious)]
#[case(KeyCode::BackTab, KeyModifiers::SHIFT, Action::FieldPrevious)]
#[case(KeyCode::Tab, KeyModifiers::SHIFT, Action::FieldPrevious)]
fn theme_details_open_and_focus_keys_are_press_only(
    #[case] code: KeyCode,
    #[case] modifiers: KeyModifiers,
    #[case] action: Action,
) {
    let bindings = KeyBindings::default();
    for context in [
        InputContext::Browse,
        InputContext::Colors,
        InputContext::ThemeInfo,
        InputContext::Preset,
    ] {
        if context == InputContext::Preset
            && !matches!(action, Action::FieldNext | Action::FieldPrevious)
        {
            continue;
        }
        let mut key = KeyEvent::new(code, modifiers);
        assert_eq!(bindings.action_for_key(key, context), Some(action));
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            key.kind = kind;
            assert_eq!(bindings.action_for_key(key, context), None);
        }
    }
    for character in [' ', 'p', 'x'] {
        assert_eq!(
            bindings.action_for_key(
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
                InputContext::Colors
            ),
            None
        );
    }
}

#[rstest]
#[case(KeyCode::Char('Z'), KeyModifiers::NONE)]
#[case(KeyCode::Char('Z'), KeyModifiers::SHIFT)]
#[case(KeyCode::Char('z'), KeyModifiers::SHIFT)]
fn maximize_key_is_literal_in_text_and_unavailable_in_temporary_views(
    #[case] code: KeyCode,
    #[case] modifiers: KeyModifiers,
) {
    let bindings = KeyBindings::default();
    let key = KeyEvent::new(code, modifiers);
    for context in [
        InputContext::Query,
        InputContext::FilterName,
        InputContext::DialogName,
    ] {
        assert_eq!(
            bindings.action_for_key(key, context),
            Some(Action::Insert('Z'))
        );
    }
    for context in [
        InputContext::FilterChoice,
        InputContext::Saved,
        InputContext::DialogConfirm,
        InputContext::Preset,
        InputContext::Copy,
        InputContext::Help,
    ] {
        assert_eq!(bindings.action_for_key(key, context), None);
    }
}

#[rstest]
#[case(KeyCode::Char('G'), KeyModifiers::NONE)]
#[case(KeyCode::Char('G'), KeyModifiers::SHIFT)]
#[case(KeyCode::Char('g'), KeyModifiers::SHIFT)]
fn shifted_last_keys_resolve_to_the_same_action(
    #[case] code: KeyCode,
    #[case] modifiers: KeyModifiers,
) {
    assert_eq!(
        KeyBindings::default().action_for_key(KeyEvent::new(code, modifiers), InputContext::Browse),
        Some(Action::Last),
    );
}

#[rstest]
#[case(KeyCode::Char('R'), KeyModifiers::NONE)]
#[case(KeyCode::Char('R'), KeyModifiers::SHIFT)]
#[case(KeyCode::Char('r'), KeyModifiers::SHIFT)]
fn uppercase_r_is_literal_in_text_and_resets_only_browse(
    #[case] code: KeyCode,
    #[case] modifiers: KeyModifiers,
) {
    let bindings = KeyBindings::default();
    let key = KeyEvent::new(code, modifiers);
    for context in [
        InputContext::Query,
        InputContext::FilterName,
        InputContext::DialogName,
    ] {
        assert_eq!(
            bindings.action_for_key(key, context),
            Some(Action::Insert('R'))
        );
    }
    assert_eq!(
        bindings.action_for_key(key, InputContext::Browse),
        Some(Action::ClearFilters)
    );
    assert_eq!(
        bindings.action_for_key(key, InputContext::FilterChoice),
        None
    );
}

#[rstest]
#[case(KeyCode::Char('S'), KeyModifiers::NONE)]
#[case(KeyCode::Char('S'), KeyModifiers::SHIFT)]
#[case(KeyCode::Char('s'), KeyModifiers::SHIFT)]
fn shifted_save_keys_are_press_only_commands_and_literal_name_input(
    #[case] code: KeyCode,
    #[case] modifiers: KeyModifiers,
) {
    let bindings = KeyBindings::default();
    for context in [
        InputContext::Browse,
        InputContext::Saved,
        InputContext::Saved,
    ] {
        let mut key = KeyEvent::new(code, modifiers);
        assert_eq!(
            bindings.action_for_key(key, context),
            Some(Action::SaveFilter)
        );
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            key.kind = kind;
            assert_eq!(bindings.action_for_key(key, context), None);
        }
    }
    assert_eq!(
        bindings.action_for_key(KeyEvent::new(code, modifiers), InputContext::DialogName),
        Some(Action::Insert('S')),
    );
    assert_eq!(
        bindings.action_for_key(
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            InputContext::Browse
        ),
        None,
    );
}

#[rstest]
#[case(InputContext::Query)]
#[case(InputContext::FilterName)]
#[case(InputContext::DialogName)]
fn common_shortcuts_are_literal_in_text_fields(#[case] context: InputContext) {
    let bindings = KeyBindings::default();
    for character in "?RPSzyi".chars() {
        assert_eq!(
            bindings.action_for_key(
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
                context
            ),
            Some(Action::Insert(character)),
        );
    }
}

#[rstest]
#[case(KeyCode::Char('h'), KeyCode::Char('l'))]
#[case(KeyCode::Left, KeyCode::Right)]
fn choice_navigation_distinguishes_choices_from_text(
    #[case] previous: KeyCode,
    #[case] next: KeyCode,
    #[values(InputContext::FilterChoice, InputContext::Preset)] context: InputContext,
) {
    let bindings = KeyBindings::default();
    for (code, action) in [(previous, Action::ValuePrevious), (next, Action::ValueNext)] {
        assert_eq!(
            bindings.action_for_key(KeyEvent::new(code, KeyModifiers::NONE), context),
            Some(action),
        );
    }
    for context in [InputContext::FilterName, InputContext::Query] {
        for character in ['h', 'l'] {
            assert_eq!(
                bindings.action_for_key(
                    KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
                    context
                ),
                Some(Action::Insert(character)),
            );
        }
    }
}

#[rstest]
#[case(KeyCode::Home, KeyModifiers::NONE, Action::CursorFirst)]
#[case(KeyCode::End, KeyModifiers::NONE, Action::CursorLast)]
#[case(KeyCode::Left, KeyModifiers::NONE, Action::CursorPrevious)]
#[case(KeyCode::Right, KeyModifiers::NONE, Action::CursorNext)]
#[case(KeyCode::Left, KeyModifiers::CONTROL, Action::CursorWordPrevious)]
#[case(KeyCode::Char('b'), KeyModifiers::ALT, Action::CursorWordPrevious)]
#[case(KeyCode::Right, KeyModifiers::CONTROL, Action::CursorWordNext)]
#[case(KeyCode::Char('f'), KeyModifiers::ALT, Action::CursorWordNext)]
#[case(KeyCode::Backspace, KeyModifiers::NONE, Action::DeleteBefore)]
#[case(KeyCode::Delete, KeyModifiers::NONE, Action::DeleteAfter)]
#[case(KeyCode::Char('w'), KeyModifiers::CONTROL, Action::DeleteWordBefore)]
#[case(KeyCode::Backspace, KeyModifiers::ALT, Action::DeleteWordBefore)]
#[case(KeyCode::Delete, KeyModifiers::CONTROL, Action::DeleteWordAfter)]
#[case(KeyCode::Char('d'), KeyModifiers::ALT, Action::DeleteWordAfter)]
#[case(KeyCode::Char('u'), KeyModifiers::CONTROL, Action::ClearInput)]
fn text_editing_keys_resolve_in_each_text_field(
    #[case] code: KeyCode,
    #[case] modifiers: KeyModifiers,
    #[case] action: Action,
) {
    let bindings = KeyBindings::default();
    for context in [
        InputContext::Query,
        InputContext::FilterName,
        InputContext::DialogName,
    ] {
        let mut key = KeyEvent::new(code, modifiers);
        for kind in [KeyEventKind::Press, KeyEventKind::Repeat] {
            key.kind = kind;
            assert_eq!(bindings.action_for_key(key, context), Some(action));
        }
        key.kind = KeyEventKind::Release;
        assert_eq!(bindings.action_for_key(key, context), None);
    }
    for context in [
        InputContext::Browse,
        InputContext::Colors,
        InputContext::FilterChoice,
        InputContext::Saved,
        InputContext::DialogConfirm,
        InputContext::Preset,
        InputContext::Copy,
        InputContext::ThemeInfo,
        InputContext::Help,
    ] {
        assert!(!context.allows(action));
        assert_ne!(
            bindings.action_for_key(KeyEvent::new(code, modifiers), context),
            Some(action)
        );
    }
}

#[rstest]
#[case(InputContext::Browse)]
#[case(InputContext::Colors)]
#[case(InputContext::Query)]
#[case(InputContext::FilterName)]
#[case(InputContext::FilterChoice)]
#[case(InputContext::Saved)]
#[case(InputContext::DialogName)]
#[case(InputContext::DialogConfirm)]
#[case(InputContext::Preset)]
#[case(InputContext::Copy)]
#[case(InputContext::ThemeInfo)]
#[case(InputContext::Help)]
fn help_uses_f1_without_consuming_question_mark_input(#[case] context: InputContext) {
    let bindings = KeyBindings::default();
    let mut key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
    assert_eq!(
        bindings.action_for_key(key, context),
        Some(Action::ToggleHelp)
    );
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        key.kind = kind;
        assert_eq!(bindings.action_for_key(key, context), None);
    }
    for modifiers in [KeyModifiers::NONE, KeyModifiers::SHIFT] {
        assert_eq!(
            bindings.action_for_key(KeyEvent::new(KeyCode::Char('?'), modifiers), context),
            context.has_text().then_some(Action::Insert('?'))
        );
    }
}
