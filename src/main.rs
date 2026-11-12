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

use std::{
    error::Error,
    fs::File,
    io,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;

use crate::{
    app::App, marks::MarkStore, preview::custom::CustomPreview, workspace::WorkspaceStore,
};

/// Terminal Color Theme Explorer 🫏
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Use a UTF-8 file as the preview for this run (- reads stdin)
    #[arg(short = 'p', long, value_name = "FILE")]
    preview: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("asinus: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let config = config::load()?;
    let custom_preview = cli
        .preview
        .as_deref()
        .map(load_custom_preview)
        .transpose()?;
    let mut store = MarkStore::from_env()?;
    let mut workspace_store = WorkspaceStore::from_env()?;
    let (marks, marks_error) = store.load();
    let (workspace, workspace_error) = workspace_store.load();
    let read_error = marks_error
        .map(|error| format!("marks: {error}"))
        .or_else(|| workspace_error.map(|error| format!("workspace: {error}")));
    let mut app = App::default();
    app.restore(marks, workspace, read_error);
    if let Some(preview) = custom_preview {
        app.use_custom_preview(preview);
    }
    let stores = if app.read_only {
        drop((store, workspace_store));
        None
    } else {
        Some((store, workspace_store))
    };
    terminal::run(app, config, stores)?;
    Ok(())
}

fn load_custom_preview(path: &Path) -> Result<CustomPreview, Box<dyn Error>> {
    if path.as_os_str() == "-" {
        terminal::ensure_available()?;
        return preview::input::read(io::stdin().lock())
            .map_err(|error| format!("could not read preview from stdin: {error}").into());
    }
    let sample = File::open(path)
        .and_then(preview::input::read)
        .map_err(|error| format!("could not read preview file {path:?}: {error}"))?;
    terminal::ensure_available()?;
    Ok(sample)
}
