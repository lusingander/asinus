use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{
    preview::SampleSize,
    theme::{Rgb, Theme},
};

pub fn lines(theme: &Theme, size: SampleSize) -> Vec<Line<'static>> {
    let compact = size == SampleSize::Small;
    let mut lines = vec![
        Line::from(vec![
            Span::raw("Foreground "),
            swatch(theme.foreground),
            Span::raw(format!(" {} Aa0", theme.foreground)),
        ]),
        Line::from(vec![
            Span::raw("Background "),
            swatch(theme.background),
            Span::raw(format!(" {}", theme.background)),
        ]),
        Line::default(),
        Line::raw(format!(
            "{:10}Normal{:gap$}Bright",
            "",
            "",
            gap = if compact { 7 } else { 15 }
        )),
    ];
    for (index, label) in [
        "Black", "Red", "Green", "Yellow", "Blue", "Magenta", "Cyan", "White",
    ]
    .into_iter()
    .enumerate()
    {
        let normal = theme.palette.normal[index];
        let bright = theme.palette.bright[index];
        let mut spans = vec![Span::raw(format!("{label:<10}"))];
        spans.extend(color_sample(normal, !compact));
        spans.push(Span::raw("     "));
        spans.extend(color_sample(bright, !compact));
        lines.push(Line::from(spans));
    }
    if compact {
        return lines;
    }
    lines.push(Line::default());
    lines.push(Line::raw(if size == SampleSize::Large {
        "The quick brown fox jumps over the lazy dog. Read, compare, choose."
    } else {
        "The quick brown fox jumps over the lazy dog."
    }));
    if size == SampleSize::Large {
        lines.extend([
            Line::raw("0123456789  ()[]{} <>  +-*/=  :;,.!?"),
            Line::default(),
        ]);
    }
    lines.extend([
        Line::raw("Regular    AaBbCc 0123456789"),
        Line::from(vec![
            Span::raw("Bold       "),
            Span::styled("AaBbCc 0123456789", Modifier::BOLD),
        ]),
    ]);
    if size == SampleSize::Large {
        lines.extend([
            Line::from(vec![
                Span::raw("Underline  "),
                Span::styled("AaBbCc 0123456789", Modifier::UNDERLINED),
            ]),
            Line::from(vec![
                Span::raw("Reverse    "),
                Span::styled(
                    "AaBbCc 0123456789",
                    Style::default()
                        .fg(Color::from(theme.background))
                        .bg(Color::from(theme.foreground)),
                ),
            ]),
            Line::default(),
            Line::raw("ANSI backgrounds (text uses Foreground)"),
        ]);
        for (label, colors) in [
            ("Normal", theme.palette.normal),
            ("Bright", theme.palette.bright),
        ] {
            let mut spans = vec![Span::raw(format!("{label:<8}"))];
            for (index, color) in colors.into_iter().enumerate() {
                if index > 0 {
                    spans.push(Span::raw(" "));
                }
                spans.push(Span::styled(
                    " Aa0 ",
                    Style::default()
                        .fg(Color::from(theme.foreground))
                        .bg(Color::from(color)),
                ));
            }
            lines.push(Line::from(spans));
        }
    }
    lines
}

fn swatch(color: Rgb) -> Span<'static> {
    Span::styled("    ", Style::default().bg(Color::from(color)))
}

fn color_sample(color: Rgb, hex: bool) -> Vec<Span<'static>> {
    vec![
        swatch(color),
        Span::raw(" "),
        Span::styled(
            if hex {
                format!("{color} Aa0")
            } else {
                "Aa0".to_owned()
            },
            Color::from(color),
        ),
    ]
}
