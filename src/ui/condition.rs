use ratatui::text::{Line, Span};

use crate::{search::condition_key_ranges, ui::theme::UiTheme};

pub fn line(query: &str, theme: UiTheme) -> Line<'static> {
    let mut spans = Vec::new();
    let mut cursor = 0;
    for range in condition_key_ranges(query) {
        if cursor < range.start {
            spans.push(Span::raw(query[cursor..range.start].to_owned()));
        }
        spans.push(Span::styled(
            query[range.clone()].to_owned(),
            theme.role_style(theme.condition_key),
        ));
        cursor = range.end;
    }
    if cursor < query.len() {
        spans.push(Span::raw(query[cursor..].to_owned()));
    }
    Line::from(spans).style(theme.base_style())
}

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Modifier, Style};
    use rstest::rstest;

    use super::*;
    use crate::ui::widgets::wrap_line;

    #[rstest]
    #[case("catpp type:dark favorite:true ignore:false", "type:favorite:ignore:")]
    #[case(
        "brightness:very-dark bg:blue contrast:medium",
        "brightness:bg:contrast:"
    )]
    #[case(r#""bg:blue" "brightness:dark" "contrast:high" bg:gray"#, "bg:")]
    #[case("bg:unknown contrast:", "bg:contrast:")]
    #[case(r#""type:dark" "favorite:true" ignore:false"#, "ignore:")]
    #[case(r#""blue  sky" "a\"type:dark" "c\\ignore:true" type:light"#, "type:")]
    #[case("夜\ttype:dark\nignore:all", "type:ignore:")]
    #[case("prototype:dark Type:dark favorite:false", "favorite:")]
    #[case("type:unknown", "type:")]
    #[case("\"unfinished type:dark", "")]
    #[case("", "")]
    fn only_condition_keys_receive_the_independent_color(
        #[case] query: &str,
        #[case] expected_keys: &str,
    ) {
        let mut theme = UiTheme::default();
        theme.condition_key.fg = Some(Color::Magenta);
        let line = line(query, theme);
        assert_eq!(
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>(),
            query
        );
        let mut colored = String::new();
        for grapheme in line.styled_graphemes(Style::reset()) {
            assert_eq!(grapheme.style.bg, Some(Color::Reset));
            assert!(!grapheme.style.add_modifier.contains(Modifier::BOLD));
            if grapheme.style.fg == Some(Color::Magenta) {
                colored.push_str(grapheme.symbol);
            } else {
                assert_eq!(grapheme.style.fg, Some(Color::Reset));
            }
        }
        assert_eq!(colored, expected_keys);
    }

    #[rstest]
    #[case(1, "夜 ty..", "ty")]
    #[case(2, "夜 type:dark..", "type:")]
    #[case(4, "夜 type:dark ignore:false", "type:ignore:")]
    fn wrapping_and_clipping_preserve_key_styles_and_reset_the_ellipsis(
        #[case] rows: usize,
        #[case] expected: &str,
        #[case] expected_keys: &str,
    ) {
        let lines = wrap_line(
            line("夜 type:dark ignore:false", UiTheme::default()),
            7,
            rows,
        );
        assert_eq!(
            lines.iter().map(Line::to_string).collect::<String>(),
            expected
        );
        let mut colored = String::new();
        for line in lines {
            assert!(line.width() <= 7);
            for grapheme in line.styled_graphemes(Style::reset()) {
                if grapheme.style.fg == Some(Color::Green) {
                    colored.push_str(grapheme.symbol);
                }
                if grapheme.symbol == "." {
                    assert_eq!(grapheme.style.fg, Some(Color::Reset));
                }
            }
        }
        assert_eq!(colored, expected_keys);
    }
}
