use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::{
    app::{App, FilterField},
    config::Config,
    filter_criteria::{BackgroundColor, Brightness, TextContrast},
    search::{Appearance, Favorite, Ignored},
    ui::{
        layout::{HEADER_LABEL_WIDTH, header_info},
        theme::UiTheme,
        widgets::{draw_input, panel, panel_with_title, theme_label},
    },
};

pub fn draw_query(frame: &mut Frame, app: &App, area: Rect, theme: UiTheme) {
    let mut spans = vec![Span::styled(
        app.themes().len().to_string(),
        theme.heading_style(),
    )];
    if !app.filter.is_default() || app.themes().len() != app.total() {
        spans.extend([
            Span::raw(" / "),
            Span::styled(app.total().to_string(), theme.heading_style()),
        ]);
    }
    let info = header_info(area, app);
    let pin = if app.mode.is_compare() {
        None
    } else {
        app.pinned_theme()
    };
    let summary_width = if pin.is_some() {
        info.x.saturating_sub(area.x + 2)
    } else {
        area.width
    };
    let mut parts = Vec::new();
    let name = app.filter.name_query();
    if !name.is_empty() {
        parts.push(format!("Name: {name}"));
    }
    match app.filter.appearance {
        Appearance::All => {}
        value => parts.push(value.label().into()),
    }
    match app.filter.favorite {
        Favorite::All => {}
        Favorite::Only => parts.push("Favorites".into()),
        Favorite::Exclude => parts.push("Not favorite".into()),
    }
    match app.filter.ignored {
        Ignored::Hide => {}
        Ignored::Show => parts.push("Ignored included".into()),
        Ignored::Only => parts.push("Ignored only".into()),
    }
    if let Some(value) = app.filter.brightness {
        parts.push(format!("BG {}", value.label().to_ascii_lowercase()));
    }
    if let Some(value) = app.filter.background_color {
        parts.push(format!("BG {}", value.label().to_ascii_lowercase()));
    }
    if let Some(value) = app.filter.text_contrast {
        parts.push(format!("Text {}", value.label().to_ascii_lowercase()));
    }
    let state = if app
        .editor
        .as_ref()
        .is_some_and(|edit| edit.error().is_some())
    {
        "Last valid results"
    } else if app.saved.is_some() {
        "Current"
    } else {
        ""
    };
    let count_width = spans.iter().map(Span::width).sum::<usize>();
    if state != "Last valid results"
        || usize::from(summary_width) >= count_width + " themes | ".len() + state.len()
    {
        spans.push(Span::raw(" themes"));
    }
    if !state.is_empty() {
        spans.extend([Span::raw(" | "), Span::raw(state)]);
    }
    let available =
        usize::from(summary_width).saturating_sub(spans.iter().map(Span::width).sum::<usize>());
    let separator = if state.is_empty() { " | " } else { " · " };
    let separator_width = Line::raw(separator).width();
    if !parts.is_empty() && available >= separator_width + 4 {
        spans.push(Span::raw(separator));
        spans.push(Span::raw(theme_label(
            &parts.join(" · "),
            available - separator_width,
        )));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(theme.base_style()),
        Rect::new(area.x, area.y, summary_width, 1),
    );
    if let Some(pin) = pin {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::raw(format!(
                    "{:<width$}",
                    "Pin",
                    width = usize::from(HEADER_LABEL_WIDTH)
                )),
                Span::styled(
                    theme_label(
                        pin.name,
                        usize::from(info.width.saturating_sub(HEADER_LABEL_WIDTH)),
                    ),
                    theme.heading_style(),
                ),
            ]))
            .style(theme.base_style()),
            info,
        );
    }
}

pub fn draw_editor(frame: &mut Frame, app: &App, config: &Config, area: Rect, preview: bool) {
    let theme = config.ui_theme;
    if app.filters().is_some() {
        draw_filters(frame, app, config, area, preview);
        return;
    }
    let Some(edit) = app.query_editor() else {
        return;
    };
    let block = editor_panel("Search", edit.error.as_deref(), area.width, theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let width = inner.width.saturating_sub(2);
    frame.render_widget(
        Paragraph::new("/ ").style(theme.active_style()),
        Rect::new(inner.x, inner.y, 2, 1),
    );
    draw_input(
        frame,
        &edit.input,
        Rect::new(inner.x + 2, inner.y, width, 1),
        theme.base_style(),
        app.help.is_none(),
    );
}

fn editor_panel(name: &str, error: Option<&str>, width: u16, theme: UiTheme) -> Block<'static> {
    let Some(error) = error else {
        return panel(format!("{name} · Draft"), theme);
    };
    let prefix = format!("{name} · ");
    let available = usize::from(width).saturating_sub(Line::raw(&prefix).width());
    panel_with_title(
        Line::from(vec![
            Span::styled(prefix, theme.heading_style()),
            Span::styled(
                theme_label(error, available),
                theme.heading_style().fg(theme.error),
            ),
        ]),
        theme,
    )
}

fn draw_current_label(frame: &mut Frame, app: &App, inner: Rect, row: u16, theme: UiTheme) {
    let width = inner.width;
    let current_label = app.theme().map_or_else(
        || "No theme selected".to_owned(),
        |theme| {
            format!(
                "Current: {}",
                theme_label(theme.name, usize::from(width.saturating_sub(9)))
            )
        },
    );
    frame.render_widget(
        Paragraph::new(current_label).style(theme.heading_style()),
        Rect::new(inner.x, inner.y + row, width, 1),
    );
}

fn draw_filters(frame: &mut Frame, app: &App, config: &Config, area: Rect, preview: bool) {
    let Some(edit) = app.filters() else {
        return;
    };
    let theme = config.ui_theme;
    let block = editor_panel("Filters", edit.error.as_deref(), area.width, theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let label_x = inner.x;
    let label_width = 16;
    let value_x = label_x + label_width;
    let value_width = inner.right().saturating_sub(value_x + 1);
    for (row, label, field) in [
        (0, "Name", FilterField::Name),
        (1, "Appearance", FilterField::Appearance),
        (2, "Favorite", FilterField::Favorite),
        (3, "Ignored", FilterField::Ignored),
        (5, "BG brightness", FilterField::Brightness),
        (6, "BG color", FilterField::BackgroundColor),
        (7, "Text contrast", FilterField::TextContrast),
    ] {
        let y = inner.y + row;
        let focused = edit.field == field;
        frame.render_widget(
            Paragraph::new(format!("{} {label}", if focused { ">" } else { " " })).style(
                if focused {
                    theme.active_style()
                } else {
                    theme.base_style()
                },
            ),
            Rect::new(label_x, y, label_width, 1),
        );
        let area = Rect::new(value_x, y, value_width, 1);
        if field == FilterField::Name {
            if edit.input.value().is_empty() && !focused {
                frame.render_widget(Paragraph::new("All names"), area);
            } else {
                draw_input(
                    frame,
                    &edit.input,
                    area,
                    theme.base_style(),
                    focused && app.help.is_none(),
                );
            }
        } else {
            let value = match field {
                FilterField::Appearance => edit.appearance.label(),
                FilterField::Favorite => edit.favorite.label(),
                FilterField::Ignored => edit.ignored.label(),
                FilterField::Brightness => edit.brightness.map_or("All", Brightness::label),
                FilterField::BackgroundColor => {
                    edit.background_color.map_or("All", BackgroundColor::label)
                }
                FilterField::TextContrast => edit.text_contrast.map_or("All", TextContrast::label),
                FilterField::Name => unreachable!(),
            };
            frame.render_widget(
                Paragraph::new(value).style(if focused {
                    theme.heading_style()
                } else {
                    theme.base_style()
                }),
                area,
            );
        }
    }
    if !preview {
        draw_current_label(frame, app, inner, 9, theme);
    }
}
