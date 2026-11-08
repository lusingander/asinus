mod support;

use std::{fs, process::Command};

use rstest::rstest;
use termlens::Color;

#[test]
fn configured_ui_styles_are_visible_on_startup() -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    fs::write(
        &config,
        r##"
[ui.style]
base = { fg = "#D1D2D3", bg = "#101112" }
focus = { fg = "bright_cyan", modifiers = ["underline"] }
key = { bg = "default" }

[ui.style.selection]
fg = "white"
bg = "bright_white"
modifiers = []
"##,
    )
    .unwrap();
    let mut terminal = termlens::bin!(
        "asinus",
        size(80, 24),
        env_clear(),
        env("ASINUS_CONFIG_FILE", config),
        env("ASINUS_STATE_PATH", dir.path().join("marks.toml")),
        env("ASINUS_WORKSPACE_PATH", dir.path().join("workspace.toml"))
    )?;
    let screen = support::wait_for_explore(&mut terminal)?;

    let heading = screen.cell(0, 1).unwrap().style();
    assert_eq!(heading.fg, Color::Rgb(0xd1, 0xd2, 0xd3));
    assert_eq!(heading.bg, Color::Rgb(0x10, 0x11, 0x12));
    assert!(heading.bold);

    let focus = screen.cell(4, 1).unwrap().style();
    assert_eq!(focus.fg, Color::Indexed(14));
    assert_eq!(focus.bg, heading.bg);
    assert!(focus.underline);
    assert!(!focus.bold);

    let selection = screen.cell(6, 5).unwrap().style();
    assert_eq!(selection.fg, Color::Indexed(7));
    assert_eq!(selection.bg, Color::Indexed(15));
    assert!(!selection.reverse);

    let key = screen.cell(23, 1).unwrap().style();
    assert_eq!(key.fg, Color::Indexed(6));
    assert_eq!(key.bg, Color::Default);

    let preview = screen.cell(6, 31).unwrap().style();
    assert_eq!(preview.fg, Color::Rgb(0x4a, 0x45, 0x43));
    assert_eq!(preview.bg, Color::Rgb(0xf7, 0xf7, 0xf7));
    support::quit(&mut terminal)
}

#[rstest]
#[case(Some("explicit.toml"), Some("xdg"), Color::Indexed(1))]
#[case(None, Some("xdg"), Color::Indexed(2))]
#[case(None, Some(""), Color::Indexed(4))]
#[case(None, None, Color::Indexed(4))]
#[case(None, Some("absent"), Color::Default)]
fn config_path_selects_a_single_file(
    #[case] explicit: Option<&str>,
    #[case] xdg: Option<&str>,
    #[case] expected_fg: Color,
) -> termlens::Result<()> {
    let dir = tempfile::tempdir().unwrap();
    for (path, color) in [
        ("explicit.toml", "red"),
        ("xdg/asinus/config.toml", "green"),
        ("home/.config/asinus/config.toml", "blue"),
    ] {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("[ui.style.base]\nfg = '{color}'")).unwrap();
    }
    let mut terminal = termlens::bin!(
        "asinus",
        size(80, 24),
        current_dir(dir.path()),
        env_clear(),
        envs(
            explicit
                .into_iter()
                .map(|path| ("ASINUS_CONFIG_FILE", path))
        ),
        envs(xdg.into_iter().map(|path| ("XDG_CONFIG_HOME", path))),
        env("HOME", dir.path().join("home")),
        env("ASINUS_STATE_PATH", dir.path().join("marks.toml")),
        env("ASINUS_WORKSPACE_PATH", dir.path().join("workspace.toml"))
    )?;
    let screen = support::wait_for_explore(&mut terminal)?;
    assert_eq!(screen.cell(0, 1).unwrap().style().fg, expected_fg);
    support::quit(&mut terminal)
}

#[rstest]
#[case(Some(""), None, "must not be empty")]
#[case(Some("missing.toml"), None, "was not found: missing.toml")]
#[case(None, Some("[ui.theme]"), "unknown field `theme`")]
#[case(None, Some("[ui.style.foucs]"), "unknown field `foucs`")]
#[case(
    None,
    Some("[ui.style.base]\nmodifiers = []"),
    "unknown field `modifiers`"
)]
#[case(None, Some("[ui.style.focus]\nfg = 'light-cyan'"), "invalid color")]
#[case(None, Some("[ui.style.focus]\nfg = '#+12345'"), "invalid color")]
#[case(
    None,
    Some("[ui.style.focus]\nmodifiers = ['blink']"),
    "unknown variant `blink`"
)]
#[case(
    None,
    Some("[ui.style.focus]\nmodifiers = [{ bold = {} }]"),
    "expected a string"
)]
fn invalid_configuration_fails_before_initializing_state(
    #[case] explicit: Option<&str>,
    #[case] content: Option<&str>,
    #[case] reason: &str,
) {
    let dir = tempfile::tempdir().unwrap();
    let config = "xdg/asinus/config.toml";
    if let Some(content) = content {
        let path = dir.path().join(config);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    let state = dir.path().join("state");
    let output = Command::new(env!("CARGO_BIN_EXE_asinus"))
        .current_dir(dir.path())
        .env_clear()
        .envs(
            explicit
                .into_iter()
                .map(|path| ("ASINUS_CONFIG_FILE", path)),
        )
        .env("XDG_CONFIG_HOME", "xdg")
        .env("XDG_DATA_HOME", &state)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!state.exists());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains(reason), "{error}");
    if content.is_some() {
        assert!(error.contains(config), "{error}");
    }
}
