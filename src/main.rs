mod app;
mod clipboard;
mod color_details;
mod color_math;
mod config;
mod copy;
mod filter_criteria;
mod keybindings;
mod marks;
mod preview;
mod search;
mod storage;
mod terminal;
mod text_input;
mod theme;
mod ui;
mod workspace;

#[cfg(test)]
mod test_support;

use std::{error::Error, process::ExitCode};

use clap::Parser;

use crate::{app::App, marks::MarkStore, workspace::WorkspaceStore};

/// Terminal Color Theme Explorer 🫏
#[derive(Parser)]
#[command(version)]
struct Cli {}

fn main() -> ExitCode {
    Cli::parse();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("asinus: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let config = config::load()?;
    let mut store = MarkStore::from_env()?;
    let mut workspace_store = WorkspaceStore::from_env()?;
    let (marks, marks_error) = store.load();
    let (workspace, workspace_error) = workspace_store.load();
    let read_error = marks_error
        .map(|error| format!("marks: {error}"))
        .or_else(|| workspace_error.map(|error| format!("workspace: {error}")));
    let mut app = App::default();
    app.restore(marks, workspace, read_error);
    let stores = if app.read_only {
        drop((store, workspace_store));
        None
    } else {
        Some((store, workspace_store))
    };
    terminal::run(app, config, stores)?;
    Ok(())
}
