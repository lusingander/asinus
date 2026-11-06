use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::{
    app::{App, InputContext, theme_target::ThemeTarget},
    preview::{self, ContentSize, Preset, SampleSize},
    theme::Theme,
    ui::{
        layout,
        theme::UiTheme,
        widgets::{panel_with_title, theme_label, theme_title},
    },
};

pub fn draw(frame: &mut Frame, area: Rect, app: &App, theme: UiTheme) {
    let size = size(area, app, app.preview_content);
    if app.mode.is_compare() {
        let [pin, current_area] = layout::comparison(area, app.compare_direction);
        draw_preview(
            frame,
            pin,
            app.pinned_theme().expect("comparison has a pinned theme"),
            app.preset,
            size,
            Line::styled(
                theme_title(
                    ThemeTarget::Pin,
                    app.pinned_theme().map(|theme| theme.name),
                    pin.width,
                ),
                theme.heading_style(),
            ),
            theme,
        );
        draw_current_preview(frame, current_area, app, size, theme);
    } else {
        draw_current_preview(frame, area, app, size, theme);
    }
}

pub fn content_area(area: Rect) -> Rect {
    sample_area(area).inner(Margin::new(2, 1))
}

fn sample_area(area: Rect) -> Rect {
    Rect::new(
        area.x,
        area.y.saturating_add(2),
        area.width,
        area.height.saturating_sub(2),
    )
}

pub fn size(area: Rect, app: &App, content: ContentSize) -> SampleSize {
    let panes = if app.mode.is_compare() {
        layout::comparison(area, app.compare_direction)
    } else {
        [area, area]
    };
    let [a, b] = panes.map(content_area);
    content.resolve(a.width.min(b.width), a.height.min(b.height))
}

fn draw_current_preview(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    size: SampleSize,
    ui_theme: UiTheme,
) {
    let focused = app.context() == InputContext::Browse && !app.theme_list_visible();
    let heading_style = if focused {
        ui_theme.active_style()
    } else {
        ui_theme.heading_style()
    };
    let title = if app.mode.is_compare() {
        let title = theme_title(
            ThemeTarget::Current,
            app.theme().map(|theme| theme.name),
            area.width,
        );
        let (label, name) = title.split_at(ThemeTarget::Current.label().len());
        Line::from(vec![
            Span::styled(label.to_owned(), heading_style),
            Span::styled(name.to_owned(), ui_theme.heading_style()),
        ])
    } else {
        app.theme().map_or_else(
            || Line::styled("No theme selected", ui_theme.heading_style()),
            |theme| {
                Line::styled(
                    theme_label(theme.name, usize::from(area.width)),
                    heading_style,
                )
            },
        )
    };
    if let Some(theme) = app.theme() {
        draw_preview(frame, area, theme, app.preset, size, title, ui_theme);
    } else {
        let block = panel_with_title(title, ui_theme);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let message = if app.mode.is_compare() {
            "No theme selected\nNo matching themes"
        } else {
            "No matching themes"
        };
        frame.render_widget(Paragraph::new(message), inner);
    }
}

fn draw_preview(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    preset: Preset,
    size: SampleSize,
    title: Line<'static>,
    ui_theme: UiTheme,
) {
    let sample_area = sample_area(area);
    let content = content_area(area);
    let lines = preview::lines(theme, preset, size);
    frame.render_widget(panel_with_title(title, ui_theme), area);
    let style = Style::reset()
        .fg(Color::from(theme.foreground))
        .bg(Color::from(theme.background));
    frame.render_widget(Block::default().style(style), sample_area);
    frame.render_widget(Paragraph::new(lines).style(style), content);
}
