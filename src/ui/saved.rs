use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use crate::{
    app::{App, InputContext, saved::DialogKind},
    config::Config,
    ui::{
        condition, help,
        theme::UiTheme,
        widgets::{
            draw_input, draw_list, draw_modal, draw_scrollbar, panel, panel_with_title,
            theme_label, wrap_line, wrap_text,
        },
    },
};

pub fn draw(frame: &mut Frame, app: &App, left: Rect, right: Rect, theme: UiTheme) {
    let screen = app.saved.as_ref().unwrap();
    let selected = screen.list.selected();
    let items: Vec<_> = std::iter::once("Initial [built-in]")
        .chain(
            app.workspace
                .saved_filters
                .iter()
                .map(|saved| saved.name.as_str()),
        )
        .map(|name| {
            ListItem::new(format!(
                " {} ",
                theme_label(name, left.width.saturating_sub(2) as usize)
            ))
        })
        .collect();
    let count = items.len();
    let block = panel_with_title(
        Line::styled(
            "Saved filters",
            if app.context() == InputContext::Saved {
                theme.focus_style()
            } else {
                theme.heading_style()
            },
        ),
        theme,
    )
    .title_top(Line::raw(format!("{} / {}", selected.map_or(0, |i| i + 1), count)).right_aligned());
    let content = block.inner(left);
    draw_list(
        frame,
        List::new(items)
            .block(block)
            .highlight_style(theme.selection_style()),
        left,
        &screen.list,
    );
    draw_scrollbar(frame, content, count, screen.list.offset(), theme);
    let block = panel("Selected filter", theme);
    let content = block.inner(right);
    frame.render_widget(block, right);
    let choice = app.selected_filter();
    let query = choice.map_or("—", |(_, query)| query);
    let current = condition_lines(&app.filter.query(), content.width, 2, theme);
    let query_rows = usize::from(content.height)
        .saturating_sub(current.len() + 7)
        .max(1);
    let mut lines = vec![Line::raw("Condition")];
    lines.extend(condition_lines(query, content.width, query_rows, theme));
    if choice.is_some() {
        lines.push(matching_count(app.saved_matches(query), theme));
    }
    lines.push(Line::default());
    lines.push(Line::styled("Current exploration", theme.heading_style()));
    lines.push(Line::default());
    lines.push(Line::raw("Condition"));
    lines.extend(current);
    lines.push(matching_count(app.themes().len(), theme));
    frame.render_widget(Paragraph::new(lines).style(theme.base_style()), content);
}

fn condition_lines(query: &str, width: u16, rows: usize, theme: UiTheme) -> Vec<Line<'static>> {
    wrap_line(condition::line(query, theme), width.saturating_sub(2), rows)
        .into_iter()
        .map(|mut line| {
            line.spans.insert(0, Span::raw("  "));
            line
        })
        .collect()
}

fn matching_count(count: usize, theme: UiTheme) -> Line<'static> {
    Line::from(vec![
        Span::styled(count.to_string(), theme.heading_style()),
        Span::raw(" matching themes"),
    ])
}

struct FilterDialogLayout {
    modal: crate::ui::layout::ModalLayout,
    lines: Vec<Line<'static>>,
    input: Option<Rect>,
    title: &'static str,
}

fn filter_dialog(app: &App, main: Rect, theme: UiTheme) -> FilterDialogLayout {
    let dialog = app.dialog.as_ref().unwrap();
    let width = main.width.min(52).saturating_sub(4);
    let mut lines = Vec::new();
    let mut input_start = None;
    let title = match &dialog.kind {
        DialogKind::New { .. } | DialogKind::Rename { .. } => {
            let prefix = Line::styled("> Name  ", theme.focus_style());
            input_start = Some((prefix.width() as u16, lines.len() as u16));
            lines.push(prefix);
            if matches!(dialog.kind, DialogKind::New { .. }) {
                "Save filter"
            } else {
                "Rename filter"
            }
        }
        DialogKind::Delete { name, .. } => {
            lines.extend(
                wrap_text(&format!("Name: {name}"), width, 2)
                    .into_iter()
                    .map(|line| line.style(theme.heading_style())),
            );
            "Delete filter?"
        }
    };
    lines.push(Line::default());
    lines.push(Line::styled(
        if matches!(dialog.kind, DialogKind::New { .. }) {
            "Condition to save"
        } else {
            "Condition"
        },
        theme.heading_style(),
    ));
    let query = match &dialog.kind {
        DialogKind::New { query }
        | DialogKind::Rename { query, .. }
        | DialogKind::Delete { query, .. } => query,
    };
    lines.extend(condition_lines(query, width, 2, theme));
    let modal = crate::ui::layout::modal(main, 52, lines.len() as u16);
    let input = input_start.map(|(column, row)| {
        Rect::new(
            modal.content.x + column,
            modal.content.y + row,
            modal.content.width.saturating_sub(column),
            1,
        )
    });
    FilterDialogLayout {
        modal,
        lines,
        input,
        title,
    }
}

pub fn draw_dialog(frame: &mut Frame, app: &App, main: Rect, config: &Config) {
    let Some(dialog) = &app.dialog else {
        return;
    };
    let theme = config.ui_theme;
    let plan = filter_dialog(app, main, theme);
    draw_modal(frame, &plan.modal, plan.title, theme);
    frame.render_widget(
        Paragraph::new(plan.lines).style(theme.base_style()),
        plan.modal.content,
    );
    if let Some(area) = plan.input {
        draw_input(
            frame,
            &dialog.input,
            area,
            theme.base_style(),
            app.help.is_none(),
        );
    }
    frame.render_widget(
        Paragraph::new(help::modal_footer(
            app,
            plan.modal.actions.width,
            &config.keybindings,
            theme,
        )),
        plan.modal.actions,
    );
}
