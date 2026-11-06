#[cfg(test)]
mod tests;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{Action, InputContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyChord {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyChord {
    pub fn new(code: KeyCode, mut modifiers: KeyModifiers) -> Self {
        let code = match code {
            KeyCode::Char(character) if character.is_ascii_alphabetic() => {
                if modifiers.contains(KeyModifiers::SHIFT) || character.is_ascii_uppercase() {
                    modifiers.insert(KeyModifiers::SHIFT);
                    KeyCode::Char(character.to_ascii_uppercase())
                } else {
                    code
                }
            }
            KeyCode::Char('?') => {
                modifiers.remove(KeyModifiers::SHIFT);
                code
            }
            _ => code,
        };
        Self { code, modifiers }
    }

    pub fn label(self) -> String {
        let modifiers = if self.code == KeyCode::BackTab {
            self.modifiers | KeyModifiers::SHIFT
        } else {
            self.modifiers
        };
        let mut label = String::new();
        for (modifier, prefix) in [
            (KeyModifiers::CONTROL, "C-"),
            (KeyModifiers::ALT, "A-"),
            (KeyModifiers::SHIFT, "S-"),
            (KeyModifiers::SUPER, "Super-"),
            (KeyModifiers::HYPER, "Hyper-"),
            (KeyModifiers::META, "Meta-"),
        ] {
            if modifier == KeyModifiers::SHIFT
                && matches!(self.code, KeyCode::Char(character) if character.is_ascii_uppercase())
            {
                continue;
            }
            if modifiers.contains(modifier) {
                label.push_str(prefix);
            }
        }
        label.push_str(&match self.code {
            KeyCode::Char(' ') => "Space".to_owned(),
            KeyCode::Char(character) => character.to_string(),
            KeyCode::Up => "↑".to_owned(),
            KeyCode::Down => "↓".to_owned(),
            KeyCode::Left => "←".to_owned(),
            KeyCode::Right => "→".to_owned(),
            KeyCode::PageUp => "PgUp".to_owned(),
            KeyCode::PageDown => "PgDn".to_owned(),
            KeyCode::Backspace => "BS".to_owned(),
            KeyCode::BackTab => "Tab".to_owned(),
            KeyCode::F(number) => format!("F{number}"),
            other => format!("{other:?}"),
        });
        label
    }
}

pub struct KeyBindings {
    pub bindings: Vec<(KeyChord, Action)>,
    pub editing: Vec<(KeyChord, Action)>,
    pub saved: Vec<(KeyChord, Action)>,
    pub menu: Vec<(KeyChord, Action)>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        use ratatui::crossterm::event::KeyCode::{
            BackTab, Backspace, Char, Delete, Down, End, Enter, Esc, F, Home, Left, Right, Tab, Up,
        };
        use ratatui::crossterm::event::KeyModifiers as M;

        Self {
            bindings: vec![
                (KeyChord::new(Char('P'), M::NONE), Action::OpenPreset),
                (KeyChord::new(Char('y'), M::NONE), Action::OpenCopy),
                (KeyChord::new(Char('i'), M::NONE), Action::OpenThemeInfo),
                (KeyChord::new(F(1), M::NONE), Action::ToggleHelp),
                (KeyChord::new(Char('j'), M::NONE), Action::Next),
                (KeyChord::new(Down, M::NONE), Action::Next),
                (KeyChord::new(Char('k'), M::NONE), Action::Previous),
                (KeyChord::new(Up, M::NONE), Action::Previous),
                (KeyChord::new(Char('g'), M::NONE), Action::First),
                (KeyChord::new(Char('G'), M::NONE), Action::Last),
                (KeyChord::new(Char('b'), M::CONTROL), Action::PagePrevious),
                (KeyChord::new(Char('f'), M::CONTROL), Action::PageNext),
                (KeyChord::new(Char('['), M::NONE), Action::PresetPrevious),
                (KeyChord::new(Char(']'), M::NONE), Action::PresetNext),
                (KeyChord::new(Char('p'), M::NONE), Action::TogglePin),
                (KeyChord::new(Char('m'), M::NONE), Action::ToggleFavorite),
                (KeyChord::new(Char('x'), M::NONE), Action::ToggleIgnore),
                (KeyChord::new(Char('c'), M::NONE), Action::ToggleCompare),
                (KeyChord::new(Char('z'), M::NONE), Action::ToggleThemeList),
                (
                    KeyChord::new(Char('Z'), M::NONE),
                    Action::TogglePanelMaximize,
                ),
                (KeyChord::new(Char('v'), M::NONE), Action::ToggleLayout),
                (KeyChord::new(Char('d'), M::NONE), Action::ToggleColors),
                (KeyChord::new(Char('t'), M::NONE), Action::SwitchThemeTarget),
                (KeyChord::new(Tab, M::NONE), Action::FieldNext),
                (KeyChord::new(BackTab, M::SHIFT), Action::FieldPrevious),
                (KeyChord::new(BackTab, M::NONE), Action::FieldPrevious),
                (KeyChord::new(Tab, M::SHIFT), Action::FieldPrevious),
                (KeyChord::new(Char('/'), M::NONE), Action::EditQuery),
                (KeyChord::new(Char('f'), M::NONE), Action::EditFilters),
                (KeyChord::new(Esc, M::NONE), Action::Cancel),
                (KeyChord::new(Char('R'), M::NONE), Action::ClearFilters),
                (KeyChord::new(Char('S'), M::NONE), Action::SaveFilter),
                (KeyChord::new(Char('s'), M::NONE), Action::OpenSaved),
                (KeyChord::new(Char('q'), M::NONE), Action::Quit),
                (KeyChord::new(Char('c'), M::CONTROL), Action::Quit),
            ],
            menu: vec![
                (KeyChord::new(Char('t'), M::NONE), Action::SwitchThemeTarget),
                (KeyChord::new(Char('h'), M::NONE), Action::ValuePrevious),
                (KeyChord::new(Char('l'), M::NONE), Action::ValueNext),
                (KeyChord::new(Left, M::NONE), Action::ValuePrevious),
                (KeyChord::new(Right, M::NONE), Action::ValueNext),
                (KeyChord::new(Tab, M::NONE), Action::FieldNext),
                (KeyChord::new(BackTab, M::SHIFT), Action::FieldPrevious),
                (KeyChord::new(BackTab, M::NONE), Action::FieldPrevious),
                (KeyChord::new(Tab, M::SHIFT), Action::FieldPrevious),
                (KeyChord::new(Char('j'), M::NONE), Action::Next),
                (KeyChord::new(Down, M::NONE), Action::Next),
                (KeyChord::new(Char('k'), M::NONE), Action::Previous),
                (KeyChord::new(Up, M::NONE), Action::Previous),
                (KeyChord::new(Char('g'), M::NONE), Action::First),
                (KeyChord::new(Char('G'), M::NONE), Action::Last),
                (KeyChord::new(Char('f'), M::CONTROL), Action::PageNext),
                (KeyChord::new(Char('b'), M::CONTROL), Action::PagePrevious),
                (KeyChord::new(Enter, M::NONE), Action::Confirm),
                (KeyChord::new(Esc, M::NONE), Action::Cancel),
                (KeyChord::new(F(1), M::NONE), Action::ToggleHelp),
                (KeyChord::new(Char('c'), M::CONTROL), Action::Quit),
            ],
            saved: vec![
                (KeyChord::new(F(1), M::NONE), Action::ToggleHelp),
                (KeyChord::new(Char('j'), M::NONE), Action::Next),
                (KeyChord::new(Down, M::NONE), Action::Next),
                (KeyChord::new(Char('k'), M::NONE), Action::Previous),
                (KeyChord::new(Up, M::NONE), Action::Previous),
                (KeyChord::new(Char('g'), M::NONE), Action::First),
                (KeyChord::new(Char('G'), M::NONE), Action::Last),
                (KeyChord::new(Char('b'), M::CONTROL), Action::PagePrevious),
                (KeyChord::new(Char('f'), M::CONTROL), Action::PageNext),
                (KeyChord::new(Char('S'), M::NONE), Action::SaveFilter),
                (KeyChord::new(Char('e'), M::NONE), Action::RenameFilter),
                (KeyChord::new(Char('D'), M::NONE), Action::DeleteFilter),
                (KeyChord::new(Enter, M::NONE), Action::Confirm),
                (KeyChord::new(Esc, M::NONE), Action::Cancel),
                (KeyChord::new(Char('c'), M::CONTROL), Action::Quit),
            ],
            editing: vec![
                (KeyChord::new(F(1), M::NONE), Action::ToggleHelp),
                (KeyChord::new(Enter, M::NONE), Action::Confirm),
                (KeyChord::new(Esc, M::NONE), Action::Cancel),
                (KeyChord::new(Char('c'), M::CONTROL), Action::Quit),
                (KeyChord::new(Tab, M::NONE), Action::FieldNext),
                (KeyChord::new(BackTab, M::SHIFT), Action::FieldPrevious),
                (KeyChord::new(BackTab, M::NONE), Action::FieldPrevious),
                (KeyChord::new(Tab, M::SHIFT), Action::FieldPrevious),
                (KeyChord::new(Down, M::NONE), Action::Next),
                (KeyChord::new(Up, M::NONE), Action::Previous),
                (KeyChord::new(Char('f'), M::CONTROL), Action::PageNext),
                (KeyChord::new(Char('b'), M::CONTROL), Action::PagePrevious),
                (KeyChord::new(Char('h'), M::NONE), Action::ValuePrevious),
                (KeyChord::new(Char('l'), M::NONE), Action::ValueNext),
                (KeyChord::new(Left, M::NONE), Action::ValuePrevious),
                (KeyChord::new(Right, M::NONE), Action::ValueNext),
                (KeyChord::new(Left, M::NONE), Action::CursorPrevious),
                (KeyChord::new(Right, M::NONE), Action::CursorNext),
                (KeyChord::new(Left, M::CONTROL), Action::CursorWordPrevious),
                (KeyChord::new(Char('b'), M::ALT), Action::CursorWordPrevious),
                (KeyChord::new(Right, M::CONTROL), Action::CursorWordNext),
                (KeyChord::new(Char('f'), M::ALT), Action::CursorWordNext),
                (KeyChord::new(Home, M::NONE), Action::CursorFirst),
                (KeyChord::new(End, M::NONE), Action::CursorLast),
                (KeyChord::new(Backspace, M::NONE), Action::DeleteBefore),
                (KeyChord::new(Delete, M::NONE), Action::DeleteAfter),
                (
                    KeyChord::new(Char('w'), M::CONTROL),
                    Action::DeleteWordBefore,
                ),
                (KeyChord::new(Backspace, M::ALT), Action::DeleteWordBefore),
                (KeyChord::new(Delete, M::CONTROL), Action::DeleteWordAfter),
                (KeyChord::new(Char('d'), M::ALT), Action::DeleteWordAfter),
                (KeyChord::new(Char('u'), M::CONTROL), Action::ClearInput),
            ],
        }
    }
}

impl KeyBindings {
    pub fn primary_key(&self, action: Action) -> Option<KeyChord> {
        self.primary_key_in(action, InputContext::Browse)
    }

    pub fn primary_key_in(&self, action: Action, context: InputContext) -> Option<KeyChord> {
        self.keys_in(action, context).first().copied()
    }

    pub fn keys_in(&self, action: Action, context: InputContext) -> Vec<KeyChord> {
        let mut keys = Vec::new();
        for (key, _) in self.context_bindings(context) {
            if self.action_for_key(KeyEvent::new(key.code, key.modifiers), context) == Some(action)
                && !keys.contains(key)
            {
                keys.push(*key);
            }
        }
        keys
    }

    fn for_context(&self, context: InputContext) -> &[(KeyChord, Action)] {
        if matches!(
            context,
            InputContext::Help
                | InputContext::Preset
                | InputContext::Copy
                | InputContext::ThemeInfo
        ) {
            &self.menu
        } else if matches!(context, InputContext::Browse | InputContext::Colors) {
            &self.bindings
        } else if context == InputContext::Saved {
            &self.saved
        } else {
            &self.editing
        }
    }

    fn context_bindings(&self, context: InputContext) -> impl Iterator<Item = &(KeyChord, Action)> {
        let extra = if context == InputContext::ThemeInfo {
            self.bindings.as_slice()
        } else {
            &[]
        };
        self.for_context(context).iter().chain(extra)
    }

    pub fn action_for_key(&self, key: KeyEvent, context: InputContext) -> Option<Action> {
        if key.kind == KeyEventKind::Release {
            return None;
        }
        if context.has_text()
            && key.modifiers.difference(KeyModifiers::SHIFT).is_empty()
            && let KeyCode::Char(character) = key.code
        {
            let character = if key.modifiers.contains(KeyModifiers::SHIFT) {
                character.to_ascii_uppercase()
            } else {
                character
            };
            return Some(Action::Insert(character));
        }
        let chord = KeyChord::new(key.code, key.modifiers);
        self.context_bindings(context)
            .find(|(bound, action)| *bound == chord && context.allows(*action))
            .map(|(_, action)| *action)
            .filter(|action| {
                key.kind != KeyEventKind::Repeat
                    || !(matches!(
                        action,
                        Action::ToggleThemeList
                            | Action::TogglePanelMaximize
                            | Action::SwitchThemeTarget
                            | Action::ToggleColors
                            | Action::ToggleCompare
                            | Action::TogglePin
                            | Action::ToggleLayout
                            | Action::ToggleFavorite
                            | Action::ToggleHelp
                            | Action::OpenPreset
                            | Action::OpenCopy
                            | Action::OpenThemeInfo
                            | Action::EditQuery
                            | Action::EditFilters
                            | Action::ClearFilters
                            | Action::Confirm
                            | Action::Cancel
                            | Action::ToggleIgnore
                            | Action::SaveFilter
                            | Action::OpenSaved
                            | Action::RenameFilter
                            | Action::DeleteFilter
                    ) || (context == InputContext::ThemeInfo
                        && matches!(action, Action::ValueNext | Action::ValuePrevious))
                        || (matches!(
                            context,
                            InputContext::Copy
                                | InputContext::Preset
                                | InputContext::ThemeInfo
                                | InputContext::Browse
                                | InputContext::Colors
                        ) && matches!(action, Action::FieldNext | Action::FieldPrevious)))
            })
    }
}
