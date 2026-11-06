use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::{
    app::{App, InputContext},
    color_details::{ColorRelation, ColorRole},
    theme::Theme,
    ui::{
        layout::{ColorDetailsLayout, detail_columns},
        theme::UiTheme,
        widgets::{comparison_titles, draw_scrollbar, theme_title},
    },
};

pub fn draw(frame: &mut Frame, app: &App, layout: &ColorDetailsLayout, ui: UiTheme) {
    let area = layout.area;
    let compare = app.mode.is_compare();
    let themes = if compare {
        [app.pinned_theme(), app.theme()]
    } else {
        [app.color_theme(), None]
    };
    let count = if compare { 2 } else { 1 };
    let row = |offset| Rect::new(area.x, area.y + offset, area.width, 1);
    draw_header(frame, row(0), app, ui);
    let [role_col, pin_col, current_col, rgb_col] = detail_columns(row(2), compare);
    let [pin_title, current_title] = if compare {
        comparison_titles(
            themes.map(|theme| theme.map(|theme| theme.name)),
            [pin_col.width, current_col.width],
        )
    } else {
        [
            theme_title(
                app.color_details.target,
                themes[0].map(|theme| theme.name),
                pin_col.width,
            ),
            String::new(),
        ]
    };
    for (cell, label) in [
        (role_col, "Role"),
        (pin_col, pin_title.as_str()),
        (current_col, current_title.as_str()),
        (rgb_col, "RGB"),
    ] {
        frame.render_widget(Paragraph::new(label).style(ui.heading_style()), cell);
    }
    let list_start = ColorDetailsLayout::HEADER_ROWS;
    let offset = app.color_details.list.offset();
    for (index, role) in ColorRole::ALL
        .iter()
        .enumerate()
        .skip(offset)
        .take(layout.rows)
    {
        let selected =
            app.color_details.is_focused() && app.color_details.list.selected() == Some(index);
        let style = if selected {
            ui.selection_style()
        } else {
            ui.base_style()
        };
        let area = row(list_start + (index - offset) as u16);
        frame.render_widget(Block::default().style(style), area);
        let [label, a, b, relation] = detail_columns(area, compare);
        frame.render_widget(Paragraph::new(role.label()).style(style), label);
        for (cell, theme) in [a, b].into_iter().zip(themes).take(count) {
            frame.render_widget(
                Paragraph::new(Line::from(value_spans(*role, theme, style))).style(style),
                cell,
            );
        }
        let relation_text = match themes[0].zip(themes[1]).map(|(a, b)| role.relation(a, b)) {
            Some(relation) => Line::styled(
                relation_label(relation),
                if relation == ColorRelation::Different && !selected {
                    style.fg(ui.comparison_diff)
                } else {
                    style
                },
            ),
            None => Line::raw("-"),
        };
        frame.render_widget(Paragraph::new(relation_text).style(style), relation);
    }
    draw_scrollbar(
        frame,
        Rect::new(area.x, area.y + list_start, area.width, layout.rows as u16),
        ColorRole::ALL.len(),
        offset,
        ui,
    );

    let selected = app.color_details.role();
    let selected_row = list_start + layout.rows as u16 + 1;
    let [label, a, b, _] = detail_columns(row(selected_row), compare);
    frame.render_widget(
        Paragraph::new(selected.label()).style(ui.heading_style()),
        label,
    );
    let [label, _, _, _] = detail_columns(row(selected_row + 1), compare);
    let has_contrast = matches!(
        selected,
        ColorRole::Foreground | ColorRole::Normal(_) | ColorRole::Bright(_)
    );
    frame.render_widget(
        Paragraph::new(if has_contrast {
            "vs own BG"
        } else {
            "Stored value"
        }),
        label,
    );
    if layout.detail_rows == 3 && has_contrast {
        let [label, _, _, _] = detail_columns(row(selected_row + 2), compare);
        frame.render_widget(Paragraph::new("BG"), label);
    }
    for (cell, theme) in [a, b].into_iter().zip(themes).take(count) {
        draw_selected_theme(
            frame,
            Rect::new(cell.x, cell.y, cell.width, layout.detail_rows),
            selected,
            theme,
            ui,
        );
    }
}

fn relation_label(relation: ColorRelation) -> &'static str {
    match relation {
        ColorRelation::Same => "Same",
        ColorRelation::Different => "Diff",
        ColorRelation::Unspecified => "Missing",
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App, ui: UiTheme) {
    let title = Line::styled(
        "Color details",
        if app.context() == InputContext::Colors {
            ui.active_style()
        } else {
            ui.heading_style()
        },
    );
    let summary = if app.mode.is_compare() {
        let pin = app
            .pinned_theme()
            .expect("comparison requires a pinned theme");
        let full_summary = comparison_summary(pin, app.theme(), false, ui);
        if title.width() + 2 + full_summary.width() <= usize::from(area.width) {
            full_summary
        } else {
            comparison_summary(pin, app.theme(), true, ui)
        }
    } else if app.color_theme().is_none() {
        Line::raw("No theme selected")
    } else {
        Line::default()
    };
    let width = summary.width() as u16;
    frame.render_widget(
        Paragraph::new(title),
        Rect::new(area.x, area.y, area.width - width - 2, 1),
    );
    frame.render_widget(
        Paragraph::new(summary),
        Rect::new(area.right() - width, area.y, width, 1),
    );
}

fn comparison_summary(
    pin: &Theme,
    current_theme: Option<&Theme>,
    compact: bool,
    ui: UiTheme,
) -> Line<'static> {
    let Some(current_theme) = current_theme else {
        return Line::raw("No theme selected");
    };
    let mut counts = [0; 3];
    for role in ColorRole::ALL {
        counts[match role.relation(pin, current_theme) {
            ColorRelation::Same => 0,
            ColorRelation::Different => 1,
            ColorRelation::Unspecified => 2,
        }] += 1;
    }
    Line::from(vec![
        Span::raw(if compact {
            format!("RGB S{} ", counts[0])
        } else {
            format!("RGB Same {} ", counts[0])
        }),
        Span::styled(
            if compact { "D" } else { "Diff" },
            ui.base_style().fg(ui.comparison_diff),
        ),
        Span::raw(if compact {
            format!("{} M{}", counts[1], counts[2])
        } else {
            format!(" {} Missing {}", counts[1], counts[2])
        }),
    ])
}

fn value_spans(role: ColorRole, theme: Option<&Theme>, style: Style) -> Vec<Span<'static>> {
    match theme.map(|theme| role.value(theme)) {
        Some(Some(value)) => vec![
            Span::styled("  ", Style::reset().bg(Color::from(value))),
            Span::styled(format!(" {value}"), style),
        ],
        Some(None) => vec![Span::raw("Not set")],
        None => vec![Span::raw("-")],
    }
}

fn draw_selected_theme(
    frame: &mut Frame,
    area: Rect,
    role: ColorRole,
    theme: Option<&Theme>,
    ui: UiTheme,
) {
    let sample = Rect::new(area.x, area.y, area.width, 1);
    let metric = Rect::new(area.x, area.y + 1, area.width, 1);
    draw_sample(frame, sample, role, theme, area.height == 2, ui);
    let Some(theme) = theme else {
        frame.render_widget(Paragraph::new("-"), metric);
        return;
    };
    if let Some(ratio) = role.contrast(theme) {
        let value = if area.height == 2 {
            format!("{ratio:.2}:1 on {}", theme.background)
        } else {
            frame.render_widget(
                Paragraph::new(theme.background.to_string()),
                Rect::new(area.x, area.y + 2, area.width, 1),
            );
            format!("{ratio:.2}:1")
        };
        frame.render_widget(Paragraph::new(value), metric);
    } else if let Some(value) = role.value(theme) {
        frame.render_widget(Paragraph::new(value.to_string()), metric);
    }
}

fn draw_sample(
    frame: &mut Frame,
    area: Rect,
    role: ColorRole,
    theme: Option<&Theme>,
    full_sample: bool,
    ui: UiTheme,
) {
    match theme.map(|theme| (theme, role.value(theme))) {
        Some((theme, Some(value))) if role.contrast(theme).is_some() => {
            let style = Style::reset()
                .fg(Color::from(value))
                .bg(Color::from(theme.background));
            let sample = if full_sample {
                "Aa 0123 sample.txt"
            } else {
                "Aa 0123"
            };
            frame.render_widget(Block::default().style(style), area);
            frame.render_widget(Paragraph::new(sample).style(style), area);
        }
        Some((_, Some(value))) => {
            frame.render_widget(
                Paragraph::new(Span::styled("    ", Style::reset().bg(Color::from(value)))),
                area,
            );
        }
        Some((_, None)) => {
            frame.render_widget(Paragraph::new("Not specified").style(ui.base_style()), area)
        }
        None => {}
    }
}
