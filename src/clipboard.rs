use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(2);

pub fn copy(text: &str) -> Result<(), String> {
    let mut missing = Vec::new();
    for mut command in commands() {
        let program = command.get_program().to_string_lossy().into_owned();
        match run(&mut command, text, TIMEOUT) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => missing.push(program),
            Err(error) => return Err(format!("{program}: {error}")),
        }
    }
    Err(if missing.is_empty() {
        "Clipboard is unavailable for this environment".into()
    } else {
        format!("Clipboard command not found: {}", missing.join(" / "))
    })
}

fn commands() -> Vec<Command> {
    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("pbcopy");
        command.env("LC_ALL", "en_US.UTF-8");
        vec![command]
    }
    #[cfg(target_os = "linux")]
    {
        linux_commands(
            std::env::var_os("WAYLAND_DISPLAY").is_some_and(|value| !value.is_empty()),
            std::env::var_os("DISPLAY").is_some_and(|value| !value.is_empty()),
        )
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Vec::new()
    }
}

#[cfg(any(target_os = "linux", test))]
fn linux_commands(wayland: bool, x11: bool) -> Vec<Command> {
    let mut commands = Vec::new();
    if wayland {
        let mut command = Command::new("wl-copy");
        command.args(["--type", "text/plain;charset=utf-8"]);
        commands.push(command);
    }
    if x11 {
        let mut command = Command::new("xclip");
        command.args(["-selection", "clipboard", "-in", "-target", "UTF8_STRING"]);
        commands.push(command);
    }
    commands
}

fn run(command: &mut Command, text: &str, timeout: Duration) -> io::Result<()> {
    // Files avoid blocking stdin writes and waiting for EOF from clipboard owners
    // that keep running after their launcher has exited.
    let mut input = tempfile::tempfile()?;
    input.write_all(text.as_bytes())?;
    input.rewind()?;
    let mut diagnostic = tempfile::tempfile()?;
    let mut child = command
        .stdin(Stdio::from(input))
        .stdout(Stdio::null())
        .stderr(Stdio::from(diagnostic.try_clone()?))
        .spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                let detail = diagnostic_text(&mut diagnostic)?;
                return Err(io::Error::other(format!(
                    "{status}{}",
                    if detail.is_empty() {
                        String::new()
                    } else {
                        format!(" · {detail}")
                    }
                )));
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(match result {
                    Err(error) => error,
                    _ => io::Error::new(io::ErrorKind::TimedOut, "clipboard command timed out"),
                });
            }
        }
    }
}

fn diagnostic_text(file: &mut File) -> io::Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    file.take(4096).read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes)
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" "))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(false, false, vec![])]
    #[case(true, false, vec!["wl-copy"])]
    #[case(false, true, vec!["xclip"])]
    #[case(true, true, vec!["wl-copy", "xclip"])]
    fn linux_uses_available_display_protocols(
        #[case] wayland: bool,
        #[case] x11: bool,
        #[case] expected: Vec<&str>,
    ) {
        let commands = linux_commands(wayland, x11);
        assert_eq!(
            commands.iter().map(|c| c.get_program()).collect::<Vec<_>>(),
            expected
        );
        if let Some(xclip) = commands.iter().find(|c| c.get_program() == "xclip") {
            assert_eq!(
                xclip.get_args().collect::<Vec<_>>(),
                ["-selection", "clipboard", "-in", "-target", "UTF8_STRING"]
            );
        }
    }

    #[cfg(unix)]
    #[rstest]
    #[case("exit 7", io::ErrorKind::Other, "exit status")]
    #[case("exec /bin/sleep 5", io::ErrorKind::TimedOut, "timed out")]
    #[case(
        r"printf '\033[31mdenied\nEND' >&2; exit 9",
        io::ErrorKind::Other,
        "[31mdenied END"
    )]
    fn command_failures_return_without_using_the_real_clipboard(
        #[case] script: &str,
        #[case] kind: io::ErrorKind,
        #[case] message: &str,
    ) {
        let error = run(
            Command::new("/bin/sh").args(["-c", script]),
            &"example".repeat(100_000),
            Duration::from_millis(50),
        )
        .unwrap_err();
        assert_eq!(error.kind(), kind);
        assert!(error.to_string().contains(message));
        assert!(!error.to_string().chars().any(char::is_control));
    }
}
