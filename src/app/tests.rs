use rstest::rstest;

use super::*;
use crate::test_support::{comparison_in_context, insert_text};

#[rstest]
#[case(InputContext::Colors)]
#[case(InputContext::ThemeInfo)]
fn maximized_panels_keep_focus_selection_and_workspace_through_temporary_views(
    #[case] context: InputContext,
    #[values(false, true)] current_focus: bool,
    #[values(false, true)] compare: bool,
) {
    let mut app = comparison_in_context(context);
    if !compare {
        app.update(Action::ToggleCompare, 14);
    }
    app.update(Action::Next, 14);
    if current_focus {
        app.update(Action::FieldNext, 14);
    }
    let focus = app.context();
    let role = app.color_details.role();
    let theme = app.theme().unwrap().name;
    let workspace = app.workspace_snapshot(None).unwrap();
    app.workspace_action = None;
    app.update(Action::TogglePanelMaximize, 14);
    assert_eq!(app.context(), focus);
    assert_eq!(app.panel_maximized(), Some(true));
    for action in [
        Action::ToggleHelp,
        Action::OpenPreset,
        Action::OpenCopy,
        Action::EditQuery,
        Action::EditFilters,
        Action::OpenSaved,
        Action::SaveFilter,
    ] {
        app.update(action, 14);
        assert_ne!(app.context(), focus);
        app.update(Action::TogglePanelMaximize, 14);
        assert_eq!(app.panel_maximized(), Some(true));
        app.update(Action::Cancel, 14);
        assert_eq!(app.context(), focus);
        assert_eq!(app.panel_maximized(), Some(true));
    }
    assert_eq!(app.color_details.role(), role);
    assert_eq!(app.theme().unwrap().name, theme);
    assert_eq!(app.workspace_snapshot(None).unwrap(), workspace);
    assert!(app.workspace_action.is_none());
    app.update(Action::TogglePanelMaximize, 14);
    assert_eq!(app.panel_maximized(), Some(false));
    assert_eq!(app.context(), focus);
}

#[rstest]
#[case(InputContext::Colors, Action::OpenThemeInfo, Action::ToggleColors)]
#[case(InputContext::ThemeInfo, Action::ToggleColors, Action::OpenThemeInfo)]
fn panel_maximize_choice_is_shared_and_survives_close_and_reopen(
    #[case] context: InputContext,
    #[case] other_panel: Action,
    #[case] original_panel: Action,
    #[values(false, true)] compare: bool,
) {
    let mut app = comparison_in_context(context);
    if !compare {
        app.update(Action::ToggleCompare, 14);
    }
    app.update(Action::TogglePanelMaximize, 14);
    app.update(Action::FieldNext, 14);
    app.update(other_panel, 14);
    assert_eq!(app.panel_maximized(), Some(true));
    app.update(Action::TogglePanelMaximize, 14);
    app.update(original_panel, 14);
    assert_eq!(app.panel_maximized(), Some(false));
    assert_eq!(app.context(), InputContext::Browse);
    app.update(Action::TogglePanelMaximize, 14);
    app.update(other_panel, 14);
    assert_eq!(app.panel_maximized(), Some(true));
    app.update(Action::Cancel, 14);
    assert_eq!(app.panel_maximized(), Some(true));
    app.update(Action::Cancel, 14);
    assert_eq!(app.panel_maximized(), None);
    app.update(Action::TogglePanelMaximize, 14);
    app.update(original_panel, 14);
    assert_eq!(app.panel_maximized(), Some(true));
}

#[rstest]
#[case(Action::ToggleCompare)]
#[case(Action::TogglePin)]
fn panel_maximize_survives_leaving_comparison_and_is_not_restored_after_restart(
    #[case] exit: Action,
) {
    let mut app = comparison_in_context(InputContext::Colors);
    app.update(Action::TogglePanelMaximize, 14);
    app.update(Action::FieldNext, 14);
    app.update(Action::Previous, 14);
    app.update(exit, 14);
    assert_eq!(app.mode, Mode::Explore);
    app.update(Action::OpenThemeInfo, 14);
    assert_eq!(app.panel_maximized(), Some(true));
    app.update(Action::Cancel, 14);
    assert_eq!(app.panel_maximized(), Some(true));
    app.update(Action::Cancel, 14);
    if exit == Action::TogglePin {
        app.update(Action::TogglePin, 14);
    }
    app.update(Action::ToggleCompare, 14);
    app.update(Action::ToggleColors, 14);
    assert_eq!(app.panel_maximized(), Some(true));
    let mut restarted = App::default();
    restarted.restore(
        app.marks.clone(),
        app.workspace_snapshot(None).unwrap(),
        None,
    );
    restarted.update(Action::TogglePanelMaximize, 14);
    restarted.update(Action::OpenThemeInfo, 14);
    assert_eq!(restarted.panel_maximized(), Some(false));
}

#[test]
fn color_focus_moves_roles_and_blocks_theme_marking_without_changing_the_hidden_list() {
    let mut app = comparison_in_context(InputContext::Colors);
    let current_name = app.theme().unwrap().name.to_owned();
    let list = app.theme_list;
    let workspace = app.workspace_snapshot(None).unwrap();
    for action in [
        Action::TogglePin,
        Action::ToggleFavorite,
        Action::ToggleIgnore,
    ] {
        app.update(action, 4);
    }
    app.update(Action::PageNext, 4);
    assert_eq!(app.color_details.list.selected(), Some(4));
    assert_eq!(app.theme_list, list);
    assert_eq!(app.theme().unwrap().name, current_name);
    assert_eq!(app.pinned_theme().unwrap().name, "3024 Day");
    assert!(app.storage_action.is_none());
    assert_eq!(app.workspace_snapshot(None).unwrap(), workspace);
    app.update(Action::FieldPrevious, 14);
    app.update(Action::Next, 14);
    assert_ne!(app.theme().unwrap().name, current_name);
    assert_eq!(app.color_details.list.selected(), Some(4));
    app.update(Action::TogglePin, 14);
    assert!(app.color_details.is_open());
    app.update(Action::TogglePin, 14);
    assert_eq!(app.mode, Mode::Explore);
    assert!(!app.theme_list_visible());
    assert!(app.color_details.is_open());
    assert_eq!(app.color_details.list.selected(), Some(4));
}

#[rstest]
#[case(false)]
#[case(true)]
fn color_details_survive_editor_cancel_and_apply_then_close_one_level_at_a_time(
    #[case] apply: bool,
) {
    let mut app = comparison_in_context(InputContext::Colors);
    app.update(Action::Last, 4);
    let before = app.theme_list;
    app.update(Action::EditQuery, 14);
    app.update(Action::ClearInput, 14);
    insert_text(&mut app, "missing-theme-e1");
    assert!(app.theme().is_none());
    app.update(
        if apply {
            Action::Confirm
        } else {
            Action::Cancel
        },
        14,
    );
    assert_eq!(app.context(), InputContext::Colors);
    assert_eq!(
        app.color_details.role(),
        crate::color_details::ColorRole::Selection
    );
    if apply {
        assert!(app.theme().is_none());
    } else {
        assert_eq!(app.theme_list, before);
    }
    app.update(Action::ToggleHelp, 14);
    app.update(Action::ToggleColors, 14);
    app.update(Action::Cancel, 14);
    assert!(app.color_details.is_focused());
    app.update(Action::Cancel, 14);
    assert!(app.mode.is_compare());
    assert!(!app.color_details.is_open());
    app.update(Action::ToggleColors, 14);
    assert_eq!(
        app.color_details.role(),
        crate::color_details::ColorRole::Selection
    );
    app.update(Action::ToggleCompare, 14);
    assert_eq!(app.mode, Mode::Explore);
    assert!(!app.theme_list_visible());
    assert!(app.color_details.is_open());
    app.update(Action::Cancel, 14);
    assert!(!app.color_details.is_open());
}

#[test]
fn reset_keeps_the_current_theme_and_only_changes_confirmed_conditions() {
    let mut app = App::default();
    app.update(Action::EditQuery, 14);
    app.update(Action::ClearInput, 14);
    insert_text(&mut app, "catpp");
    app.update(Action::Confirm, 14);
    assert_eq!(app.themes().len(), 8);
    app.update(Action::Cancel, 14);
    assert_eq!(app.themes().len(), 8);
    app.update(Action::ClearFilters, 14);
    assert_eq!(app.themes().len(), 348);
    assert_eq!(app.theme().unwrap().name, "Catppuccin Frappe");
    app.update(Action::Cancel, 14);
    assert!(!app.quit);
    assert_eq!(app.theme().unwrap().name, "Catppuccin Frappe");
    app.update(Action::EditFilters, 14);
    app.update(Action::ValueNext, 14);
    app.update(Action::ClearFilters, 14);
    assert_eq!(app.themes().len(), 298);
}

#[rstest]
#[case(InputContext::Query)]
#[case(InputContext::FilterChoice)]
fn editors_block_background_changes(#[case] context: InputContext) {
    let mut app = comparison_in_context(context);
    let session = app.workspace_snapshot(None).unwrap();
    for action in [
        Action::SaveFilter,
        Action::ToggleFavorite,
        Action::ToggleIgnore,
        Action::TogglePin,
        Action::ToggleCompare,
        Action::ToggleThemeList,
        Action::PresetNext,
    ] {
        app.update(action, 14);
    }
    assert_eq!(app.workspace_snapshot(None).unwrap(), session);
    assert_eq!(app.pinned_theme().unwrap().name, "3024 Day");
    assert!(app.marks.is_empty());
    assert!(app.storage_action.is_none());
    assert!(app.workspace_action.is_none());
    assert!(app.dialog.is_none());
    assert_eq!(app.context(), context);
}

#[test]
fn saved_screen_blocks_background_changes() {
    let mut app = comparison_in_context(InputContext::Saved);
    for action in [Action::ToggleFavorite, Action::TogglePin] {
        app.update(action, 6);
        assert_eq!(app.context(), InputContext::Saved);
        assert!(app.dialog.is_none());
        assert!(app.workspace_action.is_none());
        assert!(app.storage_action.is_none());
    }
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
fn help_blocks_background_actions(#[case] context: InputContext) {
    let mut app = comparison_in_context(context);
    let condition = app.filter.query();
    let current_name = app.theme().unwrap().name.to_owned();
    let mode = app.mode;
    let list_visible = app.theme_list_visible();
    app.update(Action::ToggleHelp, 14);
    for action in [
        Action::TogglePin,
        Action::ToggleFavorite,
        Action::ToggleIgnore,
        Action::ClearFilters,
        Action::PresetNext,
        Action::OpenPreset,
        Action::OpenCopy,
        Action::OpenThemeInfo,
        Action::SaveFilter,
        Action::EditQuery,
        Action::ToggleCompare,
        Action::ToggleThemeList,
        Action::Confirm,
    ] {
        app.update(action, 14);
    }
    assert_eq!(app.context(), InputContext::Help);
    assert_eq!(app.filter.query(), condition);
    assert_eq!(app.theme().unwrap().name, current_name);
    assert_eq!(app.pinned_theme().unwrap().name, "3024 Day");
    assert_eq!(app.mode, mode);
    assert_eq!(app.theme_list_visible(), list_visible);
    assert_eq!(app.preview_sample, Sample::BuiltIn(Preset::Palette));
    assert!(app.marks.is_empty());
    assert!(app.workspace_action.is_none());
    assert!(app.storage_action.is_none());
    assert_eq!(
        app.dialog.is_some(),
        matches!(
            context,
            InputContext::DialogConfirm | InputContext::DialogName
        )
    );
    app.update(Action::Cancel, 14);
    assert_eq!(app.context(), context);
}

#[test]
fn filter_choices_wrap_in_both_directions_and_name_fields_keep_text() {
    let mut app = App::default();
    app.update(Action::EditFilters, 14);
    for (action, count) in [
        (Action::ValuePrevious, 50),
        (Action::ValueNext, 348),
        (Action::ValueNext, 298),
        (Action::ValueNext, 50),
        (Action::ValueNext, 348),
        (Action::ValuePrevious, 50),
        (Action::ValuePrevious, 298),
        (Action::ValuePrevious, 348),
    ] {
        app.update(action, 14);
        assert_eq!(app.themes().len(), count);
    }
    app.update(Action::FieldNext, 14);
    for (action, expected) in [
        (Action::ValuePrevious, crate::search::Favorite::Exclude),
        (Action::ValueNext, crate::search::Favorite::All),
        (Action::ValueNext, crate::search::Favorite::Only),
        (Action::ValueNext, crate::search::Favorite::Exclude),
        (Action::ValueNext, crate::search::Favorite::All),
    ] {
        app.update(action, 14);
        assert_eq!(app.filters().unwrap().favorite, expected);
    }
    app.update(Action::FieldNext, 14);
    for (action, expected) in [
        (Action::ValuePrevious, crate::search::Ignored::Only),
        (Action::ValueNext, crate::search::Ignored::Hide),
        (Action::ValueNext, crate::search::Ignored::Show),
        (Action::ValueNext, crate::search::Ignored::Only),
        (Action::ValueNext, crate::search::Ignored::Hide),
    ] {
        app.update(action, 14);
        assert_eq!(app.filters().unwrap().ignored, expected);
    }
    for _ in 0..4 {
        app.update(Action::FieldNext, 14);
    }
    insert_text(&mut app, "hl");
    assert_eq!(app.filters().unwrap().input.value(), "hl");
    assert_eq!(app.filters().unwrap().input.cursor(), 2);
    app.update(Action::Cancel, 14);
    app.update(Action::EditQuery, 14);
    app.update(Action::ClearInput, 14);
    insert_text(&mut app, "hl");
    assert_eq!(app.query_editor().unwrap().input.value(), "hl");
    assert_eq!(app.query_editor().unwrap().input.cursor(), 2);
}

#[rstest]
#[case(FilterField::Brightness, "brightness", &["very-dark", "dark", "medium", "light", "very-light"])]
#[case(FilterField::BackgroundColor, "bg", &["gray", "red-pink", "orange-brown", "yellow", "green", "cyan", "blue", "purple"])]
#[case(FilterField::TextContrast, "contrast", &["low", "medium", "high"])]
fn color_choices_cycle_through_all_and_keep_the_query_in_sync(
    #[case] field: FilterField,
    #[case] key: &str,
    #[case] values: &[&str],
) {
    let mut app = App::default();
    app.update(Action::EditFilters, 14);
    while app.filters().unwrap().field != field {
        app.update(Action::FieldNext, 14);
    }
    app.update(Action::ValuePrevious, 14);
    assert_eq!(
        app.filter.query(),
        format!("ignore:false {key}:{}", values.last().unwrap())
    );
    app.update(Action::ValueNext, 14);
    assert_eq!(app.filter.query(), "ignore:false");
    for value in values {
        app.update(Action::ValueNext, 14);
        assert_eq!(app.filter.query(), format!("ignore:false {key}:{value}"));
    }
    app.update(Action::ValueNext, 14);
    assert_eq!(app.filter.query(), "ignore:false");
    app.update(Action::Cancel, 14);
    assert!(app.filter.is_default());
}

#[rstest]
#[case(Action::EditQuery)]
#[case(Action::EditFilters)]
fn word_editing_preserves_invalid_drafts_and_recovers_before_apply_or_cancel(
    #[case] open: Action,
    #[values(false, true)] apply: bool,
) {
    let mut app = App::default();
    app.update(open, 14);
    if open == Action::EditFilters {
        app.update(Action::FieldPrevious, 14);
    }
    app.update(Action::ClearInput, 14);
    insert_text(&mut app, "catpp");
    assert_eq!(app.themes().len(), 8);
    insert_text(&mut app, " unknown:value");
    let error = app.editor.as_ref().unwrap().error().unwrap().to_owned();
    let filter = app.filter.clone();
    let list = app.theme_list;
    for action in [Action::CursorWordPrevious, Action::CursorWordNext] {
        app.update(action, 14);
        assert_eq!(app.editor.as_ref().unwrap().error(), Some(error.as_str()));
        assert_eq!(app.filter, filter);
        assert_eq!(app.theme_list, list);
    }
    app.update(Action::DeleteWordBefore, 14);
    assert!(app.editor.as_ref().unwrap().error().is_none());
    assert_eq!(app.themes().len(), 8);
    app.update(
        if apply {
            Action::Confirm
        } else {
            Action::Cancel
        },
        14,
    );
    assert!(app.editor.is_none());
    assert_eq!(app.themes().len(), if apply { 8 } else { 348 });
}
