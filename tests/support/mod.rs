// Integration test binaries compile this module separately and use different helpers.
#![allow(dead_code)]

pub mod screen;

use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

use tempfile::TempDir;
use termlens::{Key, Screen, Terminal};

pub fn spawn(
    width: u16,
    height: u16,
    marks: &Path,
    workspace: &Path,
) -> termlens::Result<Terminal> {
    termlens::bin!(
        "asinus",
        size(width, height),
        env("ASINUS_STATE_PATH", marks),
        env("ASINUS_WORKSPACE_PATH", workspace)
    )
}

pub fn start(
    width: u16,
    height: u16,
    marks: &Path,
    workspace: &Path,
) -> termlens::Result<(Terminal, Screen)> {
    let mut terminal = spawn(width, height, marks, workspace)?;
    let screen = wait_for_explore(&mut terminal)?;
    Ok((terminal, screen))
}

/// Wait for an unobserved complete repaint, and assert on the returned screen.
/// The predicate must identify the result of the intended input, including the
/// last operation in a batch. Reuse the screen for further checks without input.
/// Do not mix this with waits that leave the completed-frame cursor unchanged.
pub fn wait_for(
    terminal: &mut Terminal,
    ready: impl FnMut(&Screen) -> bool,
) -> termlens::Result<Screen> {
    terminal.wait_frame(ready)
}

pub fn wait_for_explore(terminal: &mut Terminal) -> termlens::Result<Screen> {
    wait_for(terminal, |screen| {
        screen.row_text(0).contains("Explore") && screen.row_text(1).contains("348")
    })
}

pub fn quit(terminal: &mut Terminal) -> termlens::Result<()> {
    quit_with(terminal, Key::Ctrl('c'))
}

pub fn quit_with(terminal: &mut Terminal, key: Key) -> termlens::Result<()> {
    terminal.send(key)?;
    assert!(terminal.wait_exit()?.success());
    assert!(!terminal.screen().alternate_screen());
    assert!(terminal.screen().cursor_visible());
    Ok(())
}

/// Request a different size: the resize invalidates old frames before readiness
/// is checked. Reuse the previous screen when the dimensions are unchanged.
/// Wait for prior input first: its repaint can already use the new dimensions.
pub fn resize_and_wait(
    terminal: &mut Terminal,
    width: u16,
    height: u16,
    mut ready: impl FnMut(&Screen) -> bool,
) -> termlens::Result<Screen> {
    terminal.resize(width, height)?;
    wait_for(terminal, |screen| {
        screen.size() == (width, height) && ready(screen)
    })
}

pub fn edit_query(terminal: &mut Terminal, query: &str) -> termlens::Result<()> {
    terminal.send(Key::Char('/'))?;
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str(query)
}

pub fn apply_query(terminal: &mut Terminal, query: &str, count: usize) -> termlens::Result<Screen> {
    edit_query(terminal, query)?;
    let tail: String = query
        .chars()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    wait_for(terminal, |screen| {
        screen.cursor_visible() && screen.contains(&tail) && screen::matching_themes(screen, count)
    })?;
    terminal.send(Key::Enter)?;
    wait_for(terminal, |screen| {
        !screen.cursor_visible() && !screen.contains("· Draft")
    })
}

pub struct IsolatedTerminal {
    terminal: Terminal,
    _state: TempDir,
}

impl Deref for IsolatedTerminal {
    type Target = Terminal;

    fn deref(&self) -> &Terminal {
        &self.terminal
    }
}

impl DerefMut for IsolatedTerminal {
    fn deref_mut(&mut self) -> &mut Terminal {
        &mut self.terminal
    }
}

pub fn isolated(width: u16, height: u16) -> termlens::Result<IsolatedTerminal> {
    let state = tempfile::tempdir().expect("create isolated marks directory");
    let terminal = spawn(
        width,
        height,
        &state.path().join("marks.toml"),
        &state.path().join("workspace.toml"),
    )?;
    Ok(IsolatedTerminal {
        terminal,
        _state: state,
    })
}
