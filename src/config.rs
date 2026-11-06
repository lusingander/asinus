use crate::{keybindings::KeyBindings, ui::theme::UiTheme};

#[derive(Default)]
pub struct Config {
    pub ui_theme: UiTheme,
    pub keybindings: KeyBindings,
}
