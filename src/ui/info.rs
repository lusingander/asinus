use std::ops::Range;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::{App, InputContext, info::InfoSection},
    config::Config,
    theme::Theme,
    ui::{
        layout::{ThemeInfoLayout, detail_columns},
        theme::UiTheme,
        widgets::{comparison_titles, draw_scrollbar, theme_title, wrap_line, wrap_text},
    },
};

struct InfoContent {
    lines: Vec<Line<'static>>,
    ranges: Vec<Range<usize>>,
    viewport: Rect,
}

fn metric(value: f64, decimals: usize, boundaries: &[f64]) -> String {
    for digits in decimals..=15 {
        let text = format!("{value:.digits$}");
        let rounded: f64 = text.parse().unwrap();
        if boundaries.iter().all(|boundary| {
            (value < *boundary) == (rounded < *boundary)
                && (value <= *boundary) == (rounded <= *boundary)
        }) {
            return text;
        }
    }
    value.to_string()
}

fn overview_rows(theme: Option<&Theme>) -> Vec<(&'static str, Line<'static>)> {
    let text = |value: String| Line::raw(value);
    let value = |make: fn(&Theme) -> String| text(theme.map_or_else(|| "-".into(), make));
    let color = |foreground: bool| {
        theme.map_or_else(
            || Line::raw("-"),
            |theme| {
                let rgb = if foreground {
                    theme.foreground
                } else {
                    theme.background
                };
                Line::from(vec![
                    Span::styled("  ", Style::reset().bg(Color::from(rgb))),
                    Span::raw(format!(" {rgb}")),
                ])
            },
        )
    };
    vec![
        (
            "Theme",
            text(theme.map_or("No theme selected", |theme| theme.name).into()),
        ),
        (
            "Appearance",
            value(|theme| if theme.meta.is_dark { "Dark" } else { "Light" }.into()),
        ),
        ("Foreground", color(true)),
        ("Background", color(false)),
        ("", Line::default()),
        (
            "BG brightness",
            value(|theme| theme.metrics.brightness().label().into()),
        ),
        (
            "  Lightness L",
            value(|theme| {
                metric(
                    theme.metrics.background_lightness,
                    1,
                    &[25.0, 40.0, 70.0, 95.0],
                )
            }),
        ),
        (
            "BG color",
            value(|theme| theme.metrics.background_color().label().into()),
        ),
        (
            "  Chroma C",
            value(|theme| metric(theme.metrics.background_chroma, 4, &[0.015])),
        ),
        (
            "  Hue h",
            value(|theme| {
                theme.metrics.background_hue.map_or_else(
                    || "None".into(),
                    |hue| {
                        format!(
                            "{}°",
                            metric(
                                hue,
                                1,
                                &[0.0, 40.0, 85.0, 115.0, 170.0, 230.0, 285.0, 345.0, 360.0]
                            )
                        )
                    },
                )
            }),
        ),
        ("", Line::default()),
        (
            "Text contrast",
            value(|theme| theme.metrics.contrast().label().into()),
        ),
        (
            "  Ratio",
            value(|theme| format!("{}:1", metric(theme.metrics.text_contrast, 2, &[7.0, 12.0]))),
        ),
    ]
}

fn overview(app: &App, area: Rect) -> Vec<Line<'static>> {
    let compare = app.mode.is_compare();
    let cells = detail_columns(area, compare);
    let themes = if compare {
        [app.pinned_theme(), app.theme()]
    } else {
        [app.info_theme(), None]
    };
    let values = themes.map(overview_rows);
    let mut result = Vec::new();
    for (index, (label, _)) in values[0].iter().enumerate() {
        let mut parts = vec![wrap_text(label, cells[0].width, usize::MAX)];
        for column in 0..if compare { 2 } else { 1 } {
            parts.push(wrap_line(
                values[column][index].1.clone(),
                cells[column + 1].width,
                usize::MAX,
            ));
        }
        let height = parts.iter().map(Vec::len).max().unwrap_or(1);
        for row in 0..height {
            let mut spans = vec![Span::raw(" ".repeat(usize::from(cells[0].x - area.x)))];
            for (column, lines) in parts.iter().enumerate() {
                let line = lines.get(row).cloned().unwrap_or_default();
                let used = line.width();
                let style = line.style;
                spans.extend(line.spans.into_iter().map(|mut span| {
                    span.style = style.patch(span.style);
                    span
                }));
                if column + 1 < parts.len() {
                    spans.push(Span::raw(" ".repeat(
                        usize::from(cells[column + 1].x - cells[column].x).saturating_sub(used),
                    )));
                }
            }
            result.push(Line::from(spans));
        }
    }
    result
}

fn content(app: &App, layout: &ThemeInfoLayout) -> InfoContent {
    let viewport = Rect::new(
        layout.area.x,
        layout.area.y + ThemeInfoLayout::HEADER_ROWS,
        layout.area.width.saturating_sub(2),
        layout.rows as u16,
    );
    let mut ranges = Vec::new();
    let lines = if app.info_is_credits() {
        let mut lines = Vec::new();
        if let Some(theme) = app.info_theme() {
            for credit in theme.meta.credits {
                let start = lines.len();
                lines.extend(wrap_text(
                    credit.name,
                    viewport.width.saturating_sub(2),
                    usize::MAX,
                ));
                lines.extend(wrap_text(
                    credit.link,
                    viewport.width.saturating_sub(2),
                    usize::MAX,
                ));
                ranges.push(start..lines.len());
            }
            if lines.is_empty() {
                lines.push(Line::raw("No credits provided"));
            }
        } else {
            lines.push(Line::raw("No theme selected"));
        }
        lines
    } else {
        overview(app, layout.area)
    };
    InfoContent {
        lines,
        ranges,
        viewport,
    }
}

pub fn prepare(app: &mut App, layout: &ThemeInfoLayout) {
    let content = content(app, layout);
    let credits = app.info_is_credits();
    let info = app.theme_info.as_mut().unwrap();
    if credits {
        info.prepare(content.ranges, layout.rows);
    } else {
        info.prepare_overview(content.lines.len(), layout.rows);
    }
}

fn draw_header(frame: &mut Frame, app: &App, layout: &ThemeInfoLayout, ui: UiTheme) {
    let info = app.theme_info.as_ref().unwrap();
    let area = layout.area;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Theme info · ", ui.heading_style()),
            Span::styled(
                match info.section {
                    InfoSection::Overview => "Overview",
                    InfoSection::Credits => "Credits",
                },
                if app.context() == InputContext::ThemeInfo {
                    ui.active_style()
                } else {
                    ui.heading_style()
                },
            ),
        ])),
        Rect::new(area.x, area.y, area.width, 1),
    );
    let heading = Rect::new(area.x, area.y + 2, area.width, 1);
    if app.mode.is_compare() && !app.info_is_credits() {
        let [_, pin, current, _] = detail_columns(heading, true);
        let titles = comparison_titles(
            [
                app.pinned_theme().map(|theme| theme.name),
                app.theme().map(|theme| theme.name),
            ],
            [pin.width, current.width],
        );
        for (cell, title) in [pin, current].into_iter().zip(titles) {
            frame.render_widget(Paragraph::new(title).style(ui.heading_style()), cell);
        }
    } else {
        let heading = if app.info_is_credits() {
            Rect {
                width: heading.width.saturating_sub(2),
                ..heading
            }
        } else {
            detail_columns(heading, false)[1]
        };
        let title = theme_title(
            info.target,
            app.info_theme().map(|theme| theme.name),
            heading.width,
        );
        frame.render_widget(Paragraph::new(title).style(ui.heading_style()), heading);
    }
}

pub fn draw(frame: &mut Frame, app: &App, config: &Config, layout: &ThemeInfoLayout) {
    let content = content(app, layout);
    let info = app.theme_info.as_ref().unwrap();
    let ui = config.ui_theme;
    draw_header(frame, app, layout, ui);
    let offset = if app.info_is_credits() {
        info.offset
    } else {
        info.overview_offset
    };
    let selected = info.selected.and_then(|index| content.ranges.get(index));
    let lines: Vec<_> = content
        .lines
        .iter()
        .enumerate()
        .skip(offset)
        .map(|(index, line)| {
            if app.info_is_credits() {
                let padding = usize::from(content.viewport.width).saturating_sub(line.width() + 1);
                Line::styled(
                    format!(" {}{}", line, " ".repeat(padding)),
                    if info.focused && selected.is_some_and(|range| range.contains(&index)) {
                        ui.selection_style()
                    } else {
                        ui.base_style()
                    },
                )
            } else {
                line.clone()
            }
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines).style(ui.base_style()),
        content.viewport,
    );
    draw_scrollbar(
        frame,
        Rect::new(
            content.viewport.x,
            content.viewport.y,
            content.viewport.width + 1,
            content.viewport.height,
        ),
        content.lines.len(),
        offset,
        ui,
    );
}

#[cfg(test)]
mod tests {
    use ratatui::{
        Terminal,
        backend::TestBackend,
        style::{Color, Modifier},
    };
    use rstest::rstest;

    use super::*;
    use crate::{
        app::Action,
        test_support::{insert_text, render},
        ui::layout,
    };

    #[rstest]
    #[case(24.99, 1, &[25.0][..], "24.99")]
    #[case(25.0, 1, &[25.0][..], "25.0")]
    #[case(0.01501, 4, &[0.015][..], "0.01501")]
    #[case(0.01499, 4, &[0.015][..], "0.01499")]
    #[case(0.015, 4, &[0.015][..], "0.0150")]
    #[case(284.99, 1, &[285.0][..], "284.99")]
    #[case(359.99, 1, &[360.0][..], "359.99")]
    #[case(6.999, 2, &[7.0, 12.0][..], "6.999")]
    #[case(12.0, 2, &[7.0, 12.0][..], "12.00")]
    fn metric_rounding_preserves_the_side_of_classification_boundaries(
        #[case] value: f64,
        #[case] decimals: usize,
        #[case] boundaries: &[f64],
        #[case] expected: &str,
    ) {
        assert_eq!(metric(value, decimals, boundaries), expected);
    }

    #[rstest]
    #[case(60, 22)]
    #[case(80, 24)]
    #[case(120, 36)]
    fn all_credits_remain_readable_when_wrapped_scrolled_and_resized(
        #[case] width: u16,
        #[case] height: u16,
    ) {
        let mut app = App::default();
        app.update(Action::EditQuery, 14);
        app.update(Action::ClearInput, 14);
        insert_text(&mut app, "MaterialOcean");
        app.update(Action::Confirm, 14);
        app.update(Action::OpenThemeInfo, 14);
        app.update(Action::ValueNext, 14);
        let config = Config::default();
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let mut visible = Vec::new();
        loop {
            render(&mut terminal, &mut app, &config);
            let buffer = terminal.backend().buffer();
            let layout = layout::screen(buffer.area).unwrap().info(&app).unwrap();
            let content = content(&app, &layout);
            let offset = app.theme_info.as_ref().unwrap().offset;
            let selected = app.theme_info.as_ref().unwrap().selected.unwrap();
            for y in 0..content.viewport.height {
                let line: String = (content.viewport.x..content.viewport.right())
                    .map(|x| {
                        let cell = &buffer[(x, content.viewport.y + y)];
                        assert_eq!((cell.fg, cell.bg), (Color::Reset, Color::Reset));
                        assert_eq!(
                            cell.modifier.contains(Modifier::REVERSED),
                            content.ranges[selected].contains(&(offset + usize::from(y)))
                        );
                        cell.symbol()
                    })
                    .collect();
                if visible.len() == offset + usize::from(y) {
                    visible.push(line.trim().to_owned());
                }
            }
            let header: String = (layout.area.x..layout.area.right())
                .map(|x| buffer[(x, layout.area.y + 2)].symbol())
                .collect();
            assert_eq!(header.trim_end(), "Current: MaterialOcean");
            if offset + usize::from(content.viewport.height) >= content.lines.len() {
                break;
            }
            app.update(Action::Next, 1);
        }
        assert_eq!(
            visible.concat(),
            concat!(
                "kaicataldo",
                "https://github.com/kaicataldo/material.vim/blob/master/terminal-colors/iterm2/material-ocean.itermcolors",
                "fr3fou",
                "https://github.com/fr3fou"
            )
        );
        app.update(Action::Last, 1);
        terminal.backend_mut().resize(120, 36);
        terminal.resize(Rect::new(0, 0, 120, 36)).unwrap();
        render(&mut terminal, &mut app, &config);
        assert_eq!(app.theme_info.as_ref().unwrap().offset, 0);
        assert!(
            !terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .any(|cell| cell.symbol() == "┃")
        );
    }
}
