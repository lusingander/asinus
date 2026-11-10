#[cfg(test)]
mod tests;

pub mod colors;
pub mod condition;
pub mod copy;
pub mod help;
pub mod info;
pub mod layout;
pub mod preview;
pub mod saved;
pub mod search;
pub mod theme;
pub mod widgets;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
};

use crate::{
    app::{Action, App, InputContext, Notice, preview::PreviewField},
    config::Config,
    preview::{ContentSize, SampleSize},
    ui::{
        layout::{MIN_HEIGHT, MIN_WIDTH, SCREEN_MARGIN},
        theme::UiTheme,
        widgets::{
            draw_list, draw_modal, draw_scrollbar, padded_label, panel_with_title, prepare_list,
            theme_label,
        },
    },
};

/// Update layout-dependent viewports before painting the same area.
pub fn prepare(area: Rect, app: &mut App, config: &Config) {
    let Some(layout) = layout::screen(area) else {
        return;
    };
    app.sync_theme_info();
    app.sync_color_target();
    app.prepare_draw(layout.result_rows(app));
    if let Some(colors) = layout.colors(app) {
        prepare_list(
            &mut app.color_details.list,
            crate::color_details::ColorRole::ALL.len(),
            colors.rows,
        );
        if app.details_maximized {
            let max_offset = crate::color_details::ColorRole::ALL
                .len()
                .saturating_sub(colors.rows);
            let offset = app.color_details.list.offset().min(max_offset);
            *app.color_details.list.offset_mut() = offset;
        }
    }
    if app.saved.is_none() && app.filters().is_none() {
        let rows = layout
            .editor(app)
            .map(|editor| editor.visible_rows())
            .or_else(|| {
                let themes = if let Some(info) = layout.info(app) {
                    info.themes
                } else if let Some(colors) = layout.colors(app) {
                    colors.themes
                } else {
                    layout.theme_preview(app).themes
                };
                themes.map(|themes| themes.height.saturating_sub(2) as usize)
            });
        if let Some(rows) = rows {
            let length = app.themes().len();
            prepare_list(&mut app.theme_list, length, rows);
        }
    }
    let saved_count = app.workspace.saved_filters.len();
    if let Some(saved) = &mut app.saved {
        let length = saved_count + 1;
        let rows = layout.visible_rows();
        prepare_list(&mut saved.list, length, rows);
    }
    if let Some(info_layout) = layout.info(app) {
        info::prepare(app, &info_layout);
    }
    help::prepare(app, config, layout.main);
}

/// Paint the prepared application state.
pub fn draw(frame: &mut Frame, app: &App, config: &Config) {
    let area = frame.area();
    let ui_theme = config.ui_theme;
    let ui_style = ui_theme.base_style();
    frame.render_widget(Block::default().style(ui_style), area);
    let Some(layout) = layout::screen(area) else {
        draw_too_small(frame, area, config);
        return;
    };
    let resolved = layout
        .preview(app)
        .map(|area| preview::size(area, app, app.preview_content));
    draw_header(frame, app, layout.header, resolved, ui_theme);
    search::draw_query(frame, app, layout.query, ui_theme);
    frame.render_widget(
        Paragraph::new("─".repeat(layout.divider.width as usize)),
        layout.divider,
    );
    if app.saved.is_some() {
        let saved_layout = layout.saved();
        saved::draw(
            frame,
            app,
            saved_layout.list,
            saved_layout.details,
            ui_theme,
        );
    } else if let Some(editor) = layout.editor(app) {
        search::draw_editor(
            frame,
            app,
            config,
            editor.controls,
            editor.preview.is_some(),
        );
        draw_themes(
            frame,
            app,
            editor.results,
            ui_theme,
            editor.preview.is_none() && app.query_editor().is_some(),
        );
        if let Some(area) = editor.preview {
            preview::draw(frame, area, app, ui_theme);
        }
    } else if let Some(info_layout) = layout.info(app) {
        if let Some(area) = info_layout.themes {
            draw_themes(frame, app, area, ui_theme, false);
        }
        if let Some(area) = info_layout.preview {
            preview::draw(frame, area, app, ui_theme);
        }
        info::draw(frame, app, config, &info_layout);
    } else if let Some(colors) = layout.colors(app) {
        if let Some(area) = colors.themes {
            draw_themes(frame, app, area, ui_theme, false);
        }
        if let Some(area) = colors.preview {
            preview::draw(frame, area, app, ui_theme);
        }
        colors::draw(frame, app, &colors, ui_theme);
    } else {
        let preview_layout = layout.theme_preview(app);
        if let Some(area) = preview_layout.themes {
            draw_themes(frame, app, area, ui_theme, false);
        }
        preview::draw(frame, preview_layout.preview, app, ui_theme);
    }
    saved::draw_dialog(frame, app, layout.main, config);
    draw_preset_selection(frame, app, &layout, config);
    copy::draw(frame, app, config, layout.main);
    help::draw(frame, app, config, layout.main);
    frame.render_widget(
        Paragraph::new(footer(app, config, layout.footer.width)).style(ui_style),
        layout.footer,
    );
}

fn draw_header(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    resolved: Option<SampleSize>,
    theme: UiTheme,
) {
    if app.saved.is_some() {
        frame.render_widget(
            Paragraph::new("Saved filters").style(theme.heading_style()),
            area,
        );
        return;
    }
    frame.render_widget(
        Paragraph::new(app.mode.label()).style(theme.heading_style()),
        area,
    );
    let value = resolved.map_or_else(
        || app.preset.name().to_owned(),
        |size| format!("{} {}", app.preset.name(), size.name()),
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("Preview  "),
            Span::styled(value, theme.heading_style()),
        ]))
        .style(theme.base_style()),
        layout::header_info(area, app),
    );
}

fn draw_preset_selection(
    frame: &mut Frame,
    app: &App,
    screen: &layout::ScreenLayout,
    config: &Config,
) {
    let Some(selected) = app.preset_selection else {
        return;
    };
    let theme = config.ui_theme;
    let layout = layout::modal(screen.main, 52, 4);
    draw_modal(frame, &layout, "Select preview", theme);
    let inner = layout.content;
    for (index, (label, field, value)) in [
        ("Sample", PreviewField::Sample, selected.preset.name()),
        ("Content", PreviewField::Content, selected.content.name()),
    ]
    .into_iter()
    .enumerate()
    {
        let focused = selected.field == field;
        let y = inner.y + index as u16;
        frame.render_widget(
            Paragraph::new(format!("{} {label}", if focused { ">" } else { " " })).style(
                if focused {
                    theme.focus_style()
                } else {
                    theme.base_style()
                },
            ),
            Rect::new(inner.x, y, 14, 1),
        );
        frame.render_widget(
            Paragraph::new(value).style(if focused {
                theme.heading_style()
            } else {
                theme.base_style()
            }),
            Rect::new(inner.x + 14, y, inner.width.saturating_sub(14), 1),
        );
    }
    if let Some(area) = screen.preview(app) {
        let size = preview::size(area, app, selected.content);
        let (width, height) = size.dimensions();
        let label = if selected.content == ContentSize::Auto {
            format!("Auto -> {}", size.name())
        } else {
            size.name().to_owned()
        };
        frame.render_widget(
            Paragraph::new(format!("{label}: {width}x{height} content")),
            Rect::new(inner.x, inner.y + 3, inner.width, 1),
        );
    }
    frame.render_widget(
        Paragraph::new(help::modal_footer(
            app,
            layout.actions.width,
            &config.keybindings,
            theme,
        )),
        layout.actions,
    );
}

fn draw_themes(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    ui_theme: UiTheme,
    show_current_label: bool,
) {
    let filters = app.filters();
    let info_focused =
        app.editor.is_none() && app.theme_info.as_ref().is_some_and(|info| info.focused);
    let pinned_name = app.pinned_theme().map(|theme| theme.name);
    let count = app.themes().len();
    let offset = filters.map_or(0, |edit| edit.offset);
    let items = app.themes().enumerate().skip(offset).map(|(index, theme)| {
        let pinned = pinned_name == Some(theme.name);
        let selected = filters.is_none() && !info_focused && app.selected() == Some(index);
        let mark = app.marks.get(theme.name).copied();
        let pointer = if filters.is_some() && app.selected() == Some(index) {
            "• "
        } else if filters.is_some() {
            "  "
        } else {
            ""
        };
        let width = area
            .width
            .saturating_sub(if filters.is_some() { 7 } else { 5 }) as usize;
        let mut spans = vec![Span::raw(format!(
            " {pointer}{}{} ",
            if pinned { "*" } else { " " },
            mark.map_or(" ", |mark| mark.symbol())
        ))];
        spans.extend(matched_name(
            theme.name,
            width,
            Some(&app.filter),
            selected,
            ui_theme,
        ));
        spans.push(Span::raw(" "));
        ListItem::new(Line::from(spans))
    });
    let title = if filters.is_some() {
        " Matches "
    } else {
        " Themes "
    };
    let mut block = panel_with_title(
        Line::styled(
            title.trim().to_owned(),
            if app.context() == InputContext::Browse {
                ui_theme.focus_style()
            } else {
                ui_theme.heading_style()
            },
        ),
        ui_theme,
    );
    if show_current_label {
        let current_label = app.theme().map_or_else(
            || "No theme selected".to_owned(),
            |theme| {
                format!(
                    "Current: {}",
                    theme_label(
                        theme.name,
                        usize::from(area.width).saturating_sub(title.trim().len() + 11)
                    )
                )
            },
        );
        block =
            block.title_top(Line::styled(current_label, ui_theme.heading_style()).right_aligned());
    }
    let inner = block.inner(area);
    let highlight_style = if info_focused || app.context() == InputContext::Colors {
        ui_theme.base_style()
    } else {
        ui_theme.selection_style()
    };
    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style);
    if filters.is_some() {
        frame.render_widget(list, area);
    } else {
        draw_list(frame, list, area, &app.theme_list);
    }
    draw_scrollbar(
        frame,
        inner,
        count,
        filters.map_or(app.theme_list.offset(), |edit| edit.offset),
        ui_theme,
    );
    if count == 0 {
        frame.render_widget(
            Paragraph::new(padded_label("No matching themes", inner.width))
                .style(ui_theme.base_style()),
            inner,
        );
    }
}

fn matched_name(
    name: &str,
    width: usize,
    filter: Option<&crate::search::Filter>,
    selected: bool,
    ui_theme: UiTheme,
) -> Vec<Span<'static>> {
    let label = theme_label(name, width);
    let end = if label == name {
        name.len()
    } else if width < 2 {
        0
    } else {
        label.len() - 2
    };
    let ranges = filter
        .and_then(|filter| filter.name_matches(name))
        .unwrap_or_default();
    let style = if selected {
        Style::default()
    } else {
        ui_theme.search_match_style()
    };
    let mut spans = Vec::new();
    let mut cursor = 0;
    for range in &ranges {
        let start = range.start.min(end);
        let stop = range.end.min(end);
        if cursor < start {
            spans.push(Span::raw(name[cursor..start].to_owned()));
        }
        if start < stop {
            spans.push(Span::styled(name[start..stop].to_owned(), style));
        }
        cursor = stop;
        if cursor == end {
            break;
        }
    }
    if cursor < end {
        spans.push(Span::raw(name[cursor..end].to_owned()));
    }
    if end < name.len() {
        let ellipsis_style = if ranges.iter().any(|range| range.end > end) {
            style
        } else {
            Style::default()
        };
        spans.push(Span::styled(label[end..].to_owned(), ellipsis_style));
    }
    spans
}

fn footer(app: &App, config: &Config, width: u16) -> Line<'static> {
    let theme = config.ui_theme;
    let (message, role) = match &app.notice {
        Some(Notice::Message(message)) => (message.clone(), theme.notice),
        Some(Notice::Success(message)) => (message.clone(), theme.success),
        Some(Notice::Warning(message)) => (message.clone(), theme.warning),
        Some(Notice::Error(message)) => (format!("Error: {message}"), theme.error),
        Some(Notice::PinRequired) => (
            config
                .keybindings
                .primary_key(Action::TogglePin)
                .map_or_else(
                    || "Pin a theme first".to_owned(),
                    |key| format!("Pin a theme first ({})", key.label()),
                ),
            theme.warning,
        ),
        None => return help::footer(app, width, &config.keybindings, theme),
    };
    Line::styled(
        theme_label(&message, usize::from(width)),
        theme.role_style(role),
    )
}

fn draw_too_small(frame: &mut Frame, area: Rect, config: &Config) {
    let theme = config.ui_theme;
    let inner = area.inner(SCREEN_MARGIN);
    let [body, _, footer] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    let message = vec![
        Line::styled("Terminal too small", theme.heading_style()),
        Line::default(),
        Line::raw(format!("Required: {MIN_WIDTH}x{MIN_HEIGHT}")),
        Line::raw(format!("Current:  {}x{}", area.width, area.height)),
    ];
    let quit = help::action_footer(
        InputContext::Browse,
        &[(&[Action::Quit], "Quit", 0)],
        footer.width,
        &config.keybindings,
        theme,
    );
    frame.render_widget(Paragraph::new(message).style(theme.base_style()), body);
    frame.render_widget(Paragraph::new(quit).style(theme.base_style()), footer);
}
