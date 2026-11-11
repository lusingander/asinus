mod support;

use std::{fs, process::Command};

use rstest::rstest;
use support::{quit, screen, wait_for};
use termlens::{Color, Key};

#[test]
fn preview_file_uses_each_theme_and_keeps_the_builtin_selection_on_restart() -> termlens::Result<()>
{
    let dir = tempfile::tempdir().unwrap();
    let marks = dir.path().join("marks.toml");
    let workspace = dir.path().join("workspace.toml");
    let sample = dir.path().join("sample with spaces.txt");
    fs::write(&sample, "\x1b[1;31mColored sample\x1b[0m\nDefault sample\n").unwrap();
    fs::write(
        &workspace,
        "version = 1\nsaved_filters = []\n[session]\npreset = 'git'\nlayout = 'horizontal'\npreview_content = 'large'\n",
    )
    .unwrap();
    let mut terminal = termlens::bin!(
        "asinus",
        size(101, 30),
        env_clear(),
        env("ASINUS_STATE_PATH", &marks),
        env("ASINUS_WORKSPACE_PATH", &workspace),
        arg("--preview"),
        arg(&sample)
    )?;

    let initial = support::wait_for_explore(&mut terminal)?;
    assert!(initial.row_text(0).contains("Preview  Custom"));
    assert!(initial.contains("Colored sample"));
    fs::write(&sample, "Changed after startup").unwrap();
    terminal.send_str("pjjc")?;
    let compared = wait_for(&mut terminal, |s| s.find_all("Colored sample").len() == 2)?;
    assert!(screen::preview_name_matches(&compared, "Pin:", "3024 Day"));
    assert!(screen::preview_name_matches(
        &compared,
        "Current:",
        "Aardvark Blue"
    ));
    for ((row, column), expected) in compared
        .find_all("Colored sample")
        .into_iter()
        .zip([Color::Rgb(219, 45, 32), Color::Rgb(170, 52, 46)])
    {
        let style = compared.cell(row, column).unwrap().style();
        assert_eq!(style.fg, expected);
        assert!(style.bold);
    }
    for ((row, column), (fg, bg)) in compared.find_all("Default sample").into_iter().zip([
        (Color::Rgb(74, 69, 67), Color::Rgb(247, 247, 247)),
        (Color::Rgb(221, 221, 221), Color::Rgb(16, 32, 64)),
    ]) {
        let style = compared.cell(row, column).unwrap().style();
        assert_eq!((style.fg, style.bg), (fg, bg));
        assert!(!style.bold);
    }
    terminal.send(Key::Char('v'))?;
    wait_for(&mut terminal, |s| screen::comparison_layout(s, true))?;
    quit(&mut terminal)?;

    let persisted = fs::read_to_string(&workspace).unwrap();
    let value: toml::Value = toml::from_str(&persisted).unwrap();
    let session = &value["session"];
    assert_eq!(session["layout"].as_str(), Some("vertical"));
    assert_eq!(session["preset"].as_str(), Some("git"));
    assert_eq!(session["preview_content"].as_str(), Some("large"));
    assert!(!persisted.contains("sample"));
    let (mut terminal, restored) = support::start(80, 24, &marks, &workspace)?;
    assert!(restored.row_text(0).contains("Preview  Git Large"));
    quit(&mut terminal)
}

#[test]
fn switches_between_custom_and_built_in_previews() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let sample = dir.path().join("sample.txt");
    fs::write(&sample, "Custom sample body").unwrap();
    let mut terminal = termlens::bin!(
        "asinus",
        size(80, 24),
        env_clear(),
        env("ASINUS_STATE_PATH", dir.path().join("marks.toml")),
        env("ASINUS_WORKSPACE_PATH", dir.path().join("workspace.toml")),
        arg("--preview"),
        arg(&sample)
    )?;
    support::wait_for_explore(&mut terminal)?;
    terminal.send(Key::Char(']'))?;
    wait_for(&mut terminal, |s| {
        s.row_text(0).contains("Preview  Palette")
    })?;
    terminal.send(Key::Char('['))?;
    wait_for(&mut terminal, |s| s.contains("Custom sample body"))?;
    terminal.send(Key::Char('P'))?;
    let selector = wait_for(&mut terminal, |s| {
        screen::focused_field_value(s, "Sample", "Custom")
    })?;
    assert!(!selector.contains("Content"));
    terminal.send(Key::Tab)?;
    terminal.send(Key::Left)?;
    wait_for(&mut terminal, |s| {
        screen::focused_field_value(s, "Sample", "Code")
    })?;
    terminal.send(Key::Enter)?;
    wait_for(&mut terminal, |s| s.row_text(0).contains("Preview  Code"))?;
    terminal.send(Key::Char(']'))?;
    wait_for(&mut terminal, |s| s.contains("Custom sample body"))?;
    quit(&mut terminal)
}

#[rstest]
#[case(None, "could not read preview file")]
#[case(Some(vec![]), "preview input is empty")]
#[case(Some(vec![0xff]), "invalid utf-8")]
#[case(Some(vec![b'x'; 1024 * 1024 + 1]), "preview input exceeds")]
fn rejects_invalid_preview_files(#[case] input: Option<Vec<u8>>, #[case] expected: &str) {
    let dir = tempfile::tempdir().unwrap();
    let sample = dir.path().join("sample.txt");
    if let Some(input) = input {
        fs::write(&sample, input).unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_asinus"))
        .env_clear()
        .arg("--preview")
        .arg(sample)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains(expected), "{error}");
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn stdin_preview_accepts_keys_and_resizes_after_eof() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let mut terminal = termlens::Terminal::builder()
        .size(80, 24)
        .env_clear()
        .env("ASINUS_STATE_PATH", dir.path().join("marks.toml"))
        .env("ASINUS_WORKSPACE_PATH", dir.path().join("workspace.toml"))
        .env("ASINUS_BIN", env!("CARGO_BIN_EXE_asinus"))
        .args([
            "-c",
            r#"printf '\033[32mPiped sample\033[0m\n' | "$ASINUS_BIN" --preview -"#,
        ])
        .spawn("/bin/sh")?;
    let initial = support::wait_for_explore(&mut terminal)?;
    assert!(initial.row_text(0).contains("Preview  Custom"));
    let (row, column) = initial.find("Piped sample").unwrap();
    assert_eq!(
        initial.cell(row, column).unwrap().style().fg,
        Color::Rgb(1, 162, 82)
    );
    terminal.send_str("pjjc")?;
    wait_for(&mut terminal, |s| s.find_all("Piped sample").len() == 2)?;
    support::resize_and_wait(&mut terminal, 100, 30, |s| {
        s.find_all("Piped sample").len() == 2
    })?;
    quit(&mut terminal)
}

#[test]
fn stdin_preview_requires_a_terminal() {
    let output = Command::new(env!("CARGO_BIN_EXE_asinus"))
        .env_clear()
        .args(["--preview", "-"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains("the preview requires a terminal on stdout"),
        "{error}"
    );
}
