use anstyle_parse::{Params, ParamsIter, Parser, Perform};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::theme::Theme;

pub struct CustomPreview {
    lines: Vec<Line<'static>>,
}

impl CustomPreview {
    pub fn parse(input: &str) -> Self {
        let mut parser: Parser = Parser::default();
        let mut sample = SampleParser::default();
        for byte in input.bytes() {
            parser.advance(&mut sample, byte);
        }
        if !sample.text.is_empty() || !sample.line.spans.is_empty() {
            sample.finish_line();
        }
        Self {
            lines: sample.lines,
        }
    }

    pub fn lines(&self, theme: &Theme, height: u16) -> Vec<Line<'_>> {
        let resolve = |color| match color {
            Color::Indexed(index @ 0..=7) => theme.palette.normal[usize::from(index)].into(),
            Color::Indexed(index @ 8..=15) => theme.palette.bright[usize::from(index - 8)].into(),
            color => color,
        };
        self.lines
            .iter()
            .take(usize::from(height))
            .map(|line| {
                Line::from_iter(line.spans.iter().map(|span| {
                    let mut style = span.style;
                    style.fg = style.fg.map(resolve);
                    style.bg = style.bg.map(resolve);
                    Span::styled(span.content.as_ref(), style)
                }))
            })
            .collect()
    }
}

#[derive(Default)]
struct SampleParser {
    lines: Vec<Line<'static>>,
    line: Line<'static>,
    text: String,
    style: Style,
}

impl SampleParser {
    fn finish_span(&mut self) {
        if !self.text.is_empty() {
            self.line
                .spans
                .push(Span::styled(std::mem::take(&mut self.text), self.style));
        }
    }

    fn finish_line(&mut self) {
        self.finish_span();
        self.lines.push(std::mem::take(&mut self.line));
    }
}

impl Perform for SampleParser {
    fn print(&mut self, character: char) {
        if !character.is_control() {
            self.text.push(character);
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.finish_line(),
            b'\t' => self.text.push_str("    "),
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: u8) {
        if ignore || !intermediates.is_empty() || action != b'm' {
            return;
        }

        let mut style = self.style;
        let mut params = params.iter();
        while let Some(parameter) = params.next() {
            style = match *parameter {
                [0] => Style::new(),
                [1] => style.add_modifier(Modifier::BOLD),
                [2] => style.add_modifier(Modifier::DIM),
                [3] => style.add_modifier(Modifier::ITALIC),
                [4] => style.add_modifier(Modifier::UNDERLINED),
                [5] => style.add_modifier(Modifier::SLOW_BLINK),
                [6] => style.add_modifier(Modifier::RAPID_BLINK),
                [7] => style.add_modifier(Modifier::REVERSED),
                [8] => style.add_modifier(Modifier::HIDDEN),
                [9] => style.add_modifier(Modifier::CROSSED_OUT),
                [22] => style.remove_modifier(Modifier::BOLD | Modifier::DIM),
                [23] => style.remove_modifier(Modifier::ITALIC),
                [24] => style.remove_modifier(Modifier::UNDERLINED),
                [25] => style.remove_modifier(Modifier::SLOW_BLINK | Modifier::RAPID_BLINK),
                [27] => style.remove_modifier(Modifier::REVERSED),
                [28] => style.remove_modifier(Modifier::HIDDEN),
                [29] => style.remove_modifier(Modifier::CROSSED_OUT),
                [code @ 30..=37] => style.fg(Color::Indexed((code - 30) as u8)),
                [code @ 40..=47] => style.bg(Color::Indexed((code - 40) as u8)),
                [code @ 90..=97] => style.fg(Color::Indexed((code - 90 + 8) as u8)),
                [code @ 100..=107] => style.bg(Color::Indexed((code - 100 + 8) as u8)),
                [38, ref color @ ..] => {
                    extended_color(color, &mut params).map_or(style, |color| style.fg(color))
                }
                [48, ref color @ ..] => {
                    extended_color(color, &mut params).map_or(style, |color| style.bg(color))
                }
                [39] => Style { fg: None, ..style },
                [49] => Style { bg: None, ..style },
                _ => style,
            };
        }
        if style != self.style {
            self.finish_span();
            self.style = style;
        }
    }
}

fn extended_color(inline: &[u16], params: &mut ParamsIter<'_>) -> Option<Color> {
    match inline {
        [] => match params.next()? {
            [5] => match params.next()? {
                [index] => indexed(*index),
                _ => None,
            },
            [2] => match (params.next(), params.next(), params.next()) {
                (Some([red]), Some([green]), Some([blue])) => rgb(*red, *green, *blue),
                _ => None,
            },
            _ => None,
        },
        [5, index] => indexed(*index),
        [2, red, green, blue] | [2, 0, red, green, blue] => rgb(*red, *green, *blue),
        _ => None,
    }
}

fn indexed(index: u16) -> Option<Color> {
    Some(Color::Indexed(u8::try_from(index).ok()?))
}

fn rgb(red: u16, green: u16, blue: u16) -> Option<Color> {
    Some(Color::Rgb(
        u8::try_from(red).ok()?,
        u8::try_from(green).ok()?,
        u8::try_from(blue).ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::theme::THEMES;

    #[rstest]
    #[case("\x1b[31;104m", Some(1), Some(12))]
    #[case("\x1b[38;5;1;48:5:12m", Some(1), Some(12))]
    #[case("\x1b[31;104m\x1b[39m", None, Some(12))]
    #[case("\x1b[31;104m\x1b[49m", Some(1), None)]
    #[case("\x1b[31;104m\x1b[m", None, None)]
    fn resolves_palette_and_default_colors(
        #[case] sgr: &str,
        #[case] fg: Option<u8>,
        #[case] bg: Option<u8>,
    ) {
        let sample = CustomPreview::parse(&format!("{sgr}sample"));
        for theme in &THEMES[..2] {
            let lines = sample.lines(theme, 1);
            let style = lines[0].spans[0].style;
            let palette = |index: u8| match index {
                0..=7 => theme.palette.normal[usize::from(index)].into(),
                _ => theme.palette.bright[usize::from(index - 8)].into(),
            };
            assert_eq!(style.fg, fg.map(palette));
            assert_eq!(style.bg, bg.map(palette));
        }
    }

    #[test]
    fn keeps_explicit_colors_and_carries_styles_across_lines() {
        let sample = CustomPreview::parse(
            "\x1b[1;38:2::12:34:56;48;5;200m色\r\n\x1b[38;2;999;1;2mB\x1b[22mC\x1b[0mD",
        );
        let lines = sample.lines(&THEMES[0], 2);
        let styled = &lines[0].spans[0];
        assert_eq!(styled.content, "色");
        assert_eq!(styled.style.fg, Some(Color::Rgb(12, 34, 56)));
        assert_eq!(styled.style.bg, Some(Color::Indexed(200)));
        assert!(styled.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(lines[1].spans[0].style, styled.style);
        assert!(
            !lines[1].spans[1]
                .style
                .add_modifier
                .contains(Modifier::BOLD)
        );
        assert_eq!(lines[1].spans[2].style, Style::new());
    }

    #[test]
    fn keeps_visible_text_and_discards_other_terminal_sequences() {
        let sample = CustomPreview::parse(
            "A\x1b]8;;https://example.com\x1b\\link\x1b]8;;\x1b\\\x1bPqignored\x1b\\\x1b[1~\tB\x07\r\n\nC\x1b[",
        );
        let text: Vec<_> = sample
            .lines(&THEMES[0], 3)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(text, ["Alink    B", "", "C"]);
    }
}
