mod support;

use rstest::rstest;
use termlens::{Key, Screen};

fn cursor_after_catpp(screen: &Screen) -> bool {
    let (row, column, visible) = screen.cursor();
    visible && column >= 5 && screen.find_all("catpp").contains(&(row, column - 5))
}

#[rstest]
#[case('/', "Search · Draft")]
#[case('f', "Filters · Draft")]
#[case('S', "Save filter")]
fn grapheme_deletion_updates_searches_and_saved_names(
    #[case] open: char,
    #[case] title: &str,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(80, 24)?;
    support::wait_for_explore(&mut terminal)?;
    terminal.send(Key::Char(open))?;
    if open == 'f' {
        terminal.send(Key::BackTab)?;
    }
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("catpp e\u{301}界")?;
    for _ in 0..3 {
        terminal.send(Key::Backspace)?;
    }
    support::wait_for(&mut terminal, |s| {
        s.contains(title)
            && cursor_after_catpp(s)
            && if open == 'S' {
                s.contains("> Name  catpp")
            } else {
                s.row_text(1).contains("8 / 348")
            }
    })?;
    terminal.send(Key::Enter)?;
    support::wait_for(&mut terminal, |s| {
        !s.cursor_visible()
            && if open == 'S' {
                s.contains("catpp: Filter saved")
            } else {
                s.row_text(1).contains("8 / 348") && !s.contains("· Draft")
            }
    })?;
    support::quit(&mut terminal)
}

#[rstest]
#[case('/', "Search · Draft")]
#[case('f', "Filters · Draft")]
#[case('S', "Save filter")]
#[case('e', "Rename filter")]
fn word_editing_updates_searches_and_saved_names(
    #[case] open: char,
    #[case] title: &str,
    #[values(false, true)] alt: bool,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(80, 24)?;
    support::wait_for_explore(&mut terminal)?;
    if open == 'e' {
        terminal.send(Key::Char('S'))?;
        terminal.send_str("Original name")?;
        terminal.send(Key::Enter)?;
        support::wait_for(&mut terminal, |s| s.contains("Original name: Filter saved"))?;
        terminal.send(Key::Char('s'))?;
        terminal.send(Key::Down)?;
    }
    terminal.send(Key::Char(open))?;
    if open == 'f' {
        terminal.send(Key::BackTab)?;
    }
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str("catpp junk, e\u{301} 👩‍💻")?;
    if alt {
        let backspace = Key::Alt('\u{7f}');
        terminal.send(Key::Alt('b'))?;
        terminal.send(backspace)?;
        terminal.send(Key::Alt('f'))?;
        terminal.send(backspace)?;
        terminal.send(Key::Alt('b'))?;
        terminal.send(Key::Alt('d'))?;
    } else {
        terminal.send(Key::Left.ctrl())?;
        terminal.send(Key::Ctrl('w'))?;
        terminal.send(Key::Right.ctrl())?;
        terminal.send(Key::Ctrl('w'))?;
        terminal.send(Key::Left.ctrl())?;
        terminal.send(Key::Delete.ctrl())?;
    }
    terminal.send(Key::Backspace)?;
    support::wait_for(&mut terminal, |s| {
        s.contains(title)
            && cursor_after_catpp(s)
            && if matches!(open, 'S' | 'e') {
                s.contains("> Name  catpp")
            } else {
                s.row_text(1).contains("8 / 348")
            }
    })?;
    terminal.send(Key::Enter)?;
    support::wait_for(&mut terminal, |s| {
        !s.cursor_visible()
            && match open {
                'S' => s.contains("catpp: Filter saved"),
                'e' => s.contains("catpp: Filter renamed"),
                _ => s.row_text(1).contains("8 / 348") && !s.contains("· Draft"),
            }
    })?;
    support::quit(&mut terminal)
}

#[rstest]
#[case('/', "Search", "catpp?", false)]
#[case('/', "Search", "catpp? \"", true)]
#[case('f', "Filters", "catpp?", false)]
#[case('f', "Filters", "catpp? \"", true)]
#[case('S', "Save filter", "New? 界", false)]
#[case('e', "Rename filter", "New? 界", false)]
fn help_preserves_text_cursor_and_draft_after_scrolling_and_resize(
    #[case] open: char,
    #[case] title: &str,
    #[case] input: &str,
    #[case] invalid: bool,
) -> termlens::Result<()> {
    let mut terminal = support::isolated(60, 22)?;
    support::wait_for_explore(&mut terminal)?;
    if open == 'e' {
        terminal.send(Key::Char('S'))?;
        terminal.send_str("Original")?;
        terminal.send(Key::Enter)?;
        support::wait_for(&mut terminal, |s| s.contains("Original: Filter saved"))?;
        terminal.send(Key::Char('s'))?;
        terminal.send(Key::Down)?;
        support::wait_for(&mut terminal, |s| s.row_text(4).contains("2 / 2"))?;
    }
    terminal.send(Key::Char(open))?;
    if open == 'f' {
        terminal.send(Key::BackTab)?;
    }
    terminal.send(Key::Ctrl('u'))?;
    terminal.send_str(input)?;
    terminal.send(Key::Home)?;
    let before = support::wait_for(&mut terminal, |s| {
        s.contains(title)
            && s.contains(input)
            && s.cursor_visible()
            && (!invalid || s.contains("Last valid results"))
            && s.find_all(input).contains(&(s.cursor().0, s.cursor().1))
    })?;
    terminal.send(Key::F(1))?;
    let heading = format!("Help · {title}");
    let help = support::wait_for(&mut terminal, |s| {
        s.contains(&heading) && !s.cursor_visible()
    })?;
    assert!(help.row_text(21).trim_end().ends_with("Esc/F1 Close"));
    if open == '/' && !invalid {
        insta::assert_snapshot!("search_help_minimum", help.with_styles());
    }
    terminal.send_str("p?S")?;
    terminal.send(Key::Char('G'))?;
    support::wait_for(&mut terminal, |s| {
        s.contains(&heading) && s.contains("Quit the application") && !s.cursor_visible()
    })?;
    support::resize_and_wait(&mut terminal, 80, 24, |s| {
        s.contains(&heading) && !s.cursor_visible()
    })?;
    support::resize_and_wait(&mut terminal, 60, 22, |s| {
        s.contains(&heading) && !s.cursor_visible()
    })?;
    terminal.send(Key::F(1))?;
    let after = support::wait_for(&mut terminal, |s| {
        !s.contains("Help ·") && s.cursor_visible()
    })?;
    assert_eq!(after.cursor(), before.cursor());
    assert_eq!(
        after.with_styles().to_string(),
        before.with_styles().to_string()
    );
    terminal.send(Key::Esc)?;
    support::wait_for(&mut terminal, |s| !s.cursor_visible())?;
    support::quit(&mut terminal)
}
