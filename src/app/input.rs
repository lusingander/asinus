use crate::app::{
    App,
    editor::{Editor, FilterField},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Previous,
    Next,
    First,
    Last,
    PagePrevious,
    PageNext,
    PresetPrevious,
    PresetNext,
    TogglePin,
    ToggleFavorite,
    ToggleIgnore,
    ToggleCompare,
    ToggleThemeList,
    ToggleLayout,
    ToggleColors,
    TogglePanelMaximize,
    EditQuery,
    EditFilters,
    ClearFilters,
    Confirm,
    Cancel,
    FieldNext,
    FieldPrevious,
    SwitchThemeTarget,
    ValuePrevious,
    ValueNext,
    Insert(char),
    DeleteBefore,
    DeleteAfter,
    DeleteWordBefore,
    DeleteWordAfter,
    CursorPrevious,
    CursorNext,
    CursorWordPrevious,
    CursorWordNext,
    CursorFirst,
    CursorLast,
    ClearInput,
    Quit,
    SaveFilter,
    OpenSaved,
    RenameFilter,
    DeleteFilter,
    OpenPreset,
    ToggleHelp,
    OpenCopy,
    OpenThemeInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputContext {
    Browse,
    Colors,
    Query,
    FilterName,
    FilterChoice,
    Saved,
    DialogName,
    DialogConfirm,
    Preset,
    Copy,
    ThemeInfo,
    Help,
}

impl InputContext {
    pub fn has_text(self) -> bool {
        matches!(self, Self::Query | Self::FilterName | Self::DialogName)
    }

    pub fn allows(self, action: Action) -> bool {
        use crate::app::input::Action::*;
        if matches!(action, Quit | ToggleHelp) {
            return true;
        }
        if self == Self::Colors && action == SwitchThemeTarget {
            return true;
        }
        match self {
            Self::Browse | Self::Colors => {
                matches!(
                    action,
                    Previous
                        | Next
                        | First
                        | Last
                        | PagePrevious
                        | PageNext
                        | PresetPrevious
                        | PresetNext
                        | TogglePin
                        | ToggleFavorite
                        | ToggleIgnore
                        | ToggleCompare
                        | ToggleThemeList
                        | ToggleLayout
                        | ToggleColors
                        | TogglePanelMaximize
                        | FieldNext
                        | FieldPrevious
                        | EditQuery
                        | EditFilters
                        | ClearFilters
                        | Cancel
                        | SaveFilter
                        | OpenSaved
                        | OpenPreset
                        | OpenCopy
                        | OpenThemeInfo
                ) && !(self == Self::Colors
                    && matches!(action, TogglePin | ToggleFavorite | ToggleIgnore))
            }
            Self::Saved => matches!(
                action,
                Previous
                    | Next
                    | First
                    | Last
                    | PagePrevious
                    | PageNext
                    | SaveFilter
                    | RenameFilter
                    | DeleteFilter
                    | Confirm
                    | Cancel
            ),
            Self::Help => {
                matches!(
                    action,
                    Previous | Next | First | Last | PagePrevious | PageNext | Cancel
                )
            }
            Self::Preset => matches!(
                action,
                FieldNext | FieldPrevious | ValueNext | ValuePrevious | Confirm | Cancel
            ),
            Self::Copy => matches!(
                action,
                Previous | Next | SwitchThemeTarget | Confirm | Cancel
            ),
            Self::ThemeInfo => {
                Self::Colors.allows(action)
                    || matches!(
                        action,
                        ValueNext | ValuePrevious | SwitchThemeTarget | Confirm
                    )
            }
            Self::DialogConfirm => matches!(action, Confirm | Cancel),
            Self::Query | Self::FilterName | Self::FilterChoice | Self::DialogName => {
                match action {
                    Confirm | Cancel => true,
                    PagePrevious | PageNext => self != Self::DialogName,
                    Next | Previous => self == Self::Query,
                    FieldNext | FieldPrevious => {
                        matches!(self, Self::FilterName | Self::FilterChoice)
                    }
                    ValueNext | ValuePrevious => self == Self::FilterChoice,
                    Insert(_) | CursorFirst | CursorLast | CursorNext | CursorPrevious
                    | CursorWordPrevious | CursorWordNext | DeleteAfter | DeleteBefore
                    | DeleteWordBefore | DeleteWordAfter | ClearInput => self.has_text(),
                    _ => false,
                }
            }
        }
    }
}

impl App {
    pub fn action_enabled(&self, action: Action) -> bool {
        if action == Action::TogglePanelMaximize {
            return self.panel_maximized().is_some();
        }
        !self.read_only
            || !matches!(
                action,
                Action::ToggleFavorite
                    | Action::ToggleIgnore
                    | Action::SaveFilter
                    | Action::RenameFilter
                    | Action::DeleteFilter
            )
    }

    pub fn context(&self) -> InputContext {
        if self.help.is_some() {
            return InputContext::Help;
        }
        if self.copy_selection.is_some() {
            return InputContext::Copy;
        }
        if self.preview_selection.is_some() {
            return InputContext::Preset;
        }
        if let Some(dialog) = &self.dialog {
            return if dialog.has_name() {
                InputContext::DialogName
            } else {
                InputContext::DialogConfirm
            };
        }
        if self.saved.is_some() {
            return InputContext::Saved;
        }
        match &self.editor {
            None if self.theme_info.is_some() => {
                if self.theme_info.as_ref().unwrap().focused {
                    InputContext::ThemeInfo
                } else {
                    InputContext::Browse
                }
            }
            None if self.color_details.is_focused() => InputContext::Colors,
            None => InputContext::Browse,
            Some(Editor::Query(_)) => InputContext::Query,
            Some(Editor::Filters(edit)) if edit.field == FilterField::Name => {
                InputContext::FilterName
            }
            Some(Editor::Filters(_)) => InputContext::FilterChoice,
        }
    }
}
