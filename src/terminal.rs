#[cfg(test)]
mod tests;

use std::{io, time::Duration};

use ratatui::{
    DefaultTerminal,
    crossterm::{
        SynchronizedUpdate,
        event::{self, Event, KeyEvent, KeyEventKind},
    },
    layout::{Rect, Size},
};

use crate::{
    app::{Action, App, InputContext, Notice},
    clipboard,
    config::Config,
    marks::MarkStore,
    ui,
    ui::layout,
    workspace::{WorkspaceAction, WorkspaceStore},
};

pub fn run(
    mut app: App,
    config: Config,
    mut stores: Option<(MarkStore, WorkspaceStore)>,
) -> io::Result<()> {
    ratatui::run(|terminal| run_loop(terminal, &mut app, &config, stores.as_mut()))
}

fn run_loop(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    config: &Config,
    mut stores: Option<&mut (MarkStore, WorkspaceStore)>,
) -> io::Result<()> {
    // Register the resize event source before publishing the first frame so
    // a resize between that frame and the first read is not lost.
    event::poll(Duration::ZERO)?;
    while !app.quit {
        io::stdout().sync_update(|_| {
            terminal.draw(|frame| {
                ui::prepare(frame.area(), app, config);
                ui::draw(frame, app, config);
            })?;
            Ok::<(), io::Error>(())
        })??;
        if let Event::Key(key) = event::read()? {
            let size = terminal.size()?;
            handle_key(app, key, size, config);
            if let Some(request) = app.copy_action.take() {
                let result = clipboard::copy(&request.text);
                app.copy_finished(request, result);
            }
            process_browser(app, |url| open::that_detached(url));
            if let Some((marks, workspace)) = stores.as_deref_mut() {
                process_storage(app, marks);
                process_workspace(app, workspace);
            }
        }
    }
    Ok(())
}

fn process_browser(app: &mut App, open_url: impl FnOnce(&str) -> io::Result<()>) {
    if let Some(url) = app.browser_action.take()
        && let Err(error) = open_url(&url)
    {
        app.notice = Some(Notice::Error(format!("Could not open browser: {error}")));
    }
}

fn process_storage(app: &mut App, store: &mut MarkStore) {
    if let Some(change) = app.storage_action.take() {
        let mut marks = app.marks.clone();
        change.apply(&mut marks);
        match store.save(&marks) {
            Ok(()) => app.mark_saved(change),
            Err(error) => {
                app.notice = Some(Notice::Error(format!("Could not save marks: {error}")));
            }
        }
    }
}

fn process_workspace(app: &mut App, store: &mut WorkspaceStore) {
    let change = match app.workspace_action.take() {
        Some(WorkspaceAction::Session) => None,
        Some(WorkspaceAction::Filter(change)) => Some(change),
        None => return,
    };
    let result = app
        .workspace_snapshot(change.as_ref())
        .and_then(|workspace| {
            store.save(&workspace)?;
            Ok(workspace)
        });
    match result {
        Ok(workspace) => app.workspace_saved(workspace, change),
        Err(error) => {
            app.notice = Some(Notice::Error(format!("Could not save workspace: {error}")))
        }
    }
}

fn handle_key(app: &mut App, key: KeyEvent, size: Size, config: &Config) {
    if key.kind != KeyEventKind::Release {
        app.notice = None;
    }
    app.sync_theme_info();
    let layout = layout::screen(Rect::new(0, 0, size.width, size.height));
    if let Some(info_layout) = layout.as_ref().and_then(|layout| layout.info(app)) {
        ui::info::prepare(app, &info_layout);
    }
    let context = if layout.is_some() {
        app.context()
    } else {
        InputContext::Browse
    };
    if let Some(action) = config.keybindings.action_for_key(key, context)
        && (action == Action::Quit || layout.is_some())
    {
        let rows = layout.map_or(0, |layout| {
            if app.help.is_some() {
                layout.visible_rows()
            } else {
                layout.input_rows(app)
            }
        });
        app.update(action, rows);
    }
}
