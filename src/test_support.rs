//! State setup uses Actions so state and UI tests can run independently of key resolution.

use ratatui::{
    Terminal,
    backend::TestBackend,
    crossterm::event::{KeyCode, KeyModifiers},
};

use crate::{
    app::{Action, App, InputContext},
    config::Config,
    keybindings::KeyChord,
    ui,
    workspace::WorkspaceChange,
};

pub fn insert_text(app: &mut App, text: &str) {
    for character in text.chars() {
        app.update(Action::Insert(character), 14);
    }
}

pub fn comparison_in_context(context: InputContext) -> App {
    let mut app = App::default();
    for action in [
        Action::TogglePin,
        Action::Next,
        Action::ToggleThemeList,
        Action::ToggleCompare,
    ] {
        app.update(action, 14);
    }
    match context {
        InputContext::Browse => {}
        InputContext::Colors => app.update(Action::ToggleColors, 14),
        InputContext::ThemeInfo => app.update(Action::OpenThemeInfo, 14),
        InputContext::Preset | InputContext::Copy => {
            app.update(
                if context == InputContext::Preset {
                    Action::OpenPreset
                } else {
                    Action::OpenCopy
                },
                14,
            );
            app.update(
                if context == InputContext::Preset {
                    Action::ValueNext
                } else {
                    Action::Next
                },
                14,
            );
        }
        InputContext::Query => app.update(Action::EditQuery, 14),
        InputContext::FilterChoice | InputContext::FilterName => {
            app.update(Action::EditFilters, 14);
            if context == InputContext::FilterName {
                app.update(Action::FieldPrevious, 14);
            }
        }
        InputContext::DialogName => app.update(Action::SaveFilter, 14),
        InputContext::Saved | InputContext::DialogConfirm => {
            app.workspace = app
                .workspace
                .changed(&WorkspaceChange::New {
                    name: "Dark".into(),
                    query: "type:dark".into(),
                })
                .unwrap();
            app.update(Action::OpenSaved, 14);
            if context == InputContext::DialogConfirm {
                app.update(Action::Next, 14);
                app.update(Action::DeleteFilter, 14);
            }
        }
        _ => panic!("unsupported comparison context"),
    }
    app
}

pub fn custom_navigation_config() -> Config {
    let mut config = Config::default();
    config.keybindings.bindings.retain(|(_, action)| {
        !matches!(
            action,
            Action::Next
                | Action::Previous
                | Action::PresetNext
                | Action::TogglePin
                | Action::ToggleCompare
                | Action::ToggleLayout
                | Action::ToggleIgnore
                | Action::SaveFilter
                | Action::OpenSaved
                | Action::OpenThemeInfo
        )
    });
    config.keybindings.bindings.extend(
        [
            ('n', Action::Next),
            ('x', Action::Next),
            ('i', Action::Previous),
            ('o', Action::PresetNext),
            ('a', Action::TogglePin),
            ('s', Action::ToggleCompare),
            ('t', Action::ToggleLayout),
        ]
        .map(|(key, action)| {
            (
                KeyChord::new(KeyCode::Char(key), KeyModifiers::NONE),
                action,
            )
        }),
    );
    config
}

pub fn render(terminal: &mut Terminal<TestBackend>, app: &mut App, config: &Config) {
    terminal
        .draw(|frame| {
            ui::prepare(frame.area(), app, config);
            ui::draw(frame, app, config);
        })
        .unwrap();
}
