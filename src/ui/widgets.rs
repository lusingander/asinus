use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListState, Padding, Paragraph},
};
use tui_input::Input;

use crate::{
    app::theme_target::ThemeTarget,
    ui::{layout::ModalLayout, theme::UiTheme},
};

pub fn theme_label(name: &str, width: usize) -> String {
    let line = Line::raw(name);
    if line.width() <= width {
        return name.to_owned();
    }
    if width < 2 {
        return ".".repeat(width);
    }
    let mut label = String::new();
    let mut used = 0;
    for grapheme in line.styled_graphemes(Style::default()) {
        let grapheme_width = Span::raw(grapheme.symbol).width();
        if used + grapheme_width > width - 2 {
            break;
        }
        label.push_str(grapheme.symbol);
        used += grapheme_width;
    }
    label.push_str("..");
    label
}

pub fn padded_label(value: &str, width: u16) -> String {
    format!(" {} ", theme_label(value, width.saturating_sub(2) as usize))
}

fn theme_name_width(target: ThemeTarget, width: u16) -> usize {
    usize::from(width).saturating_sub(Line::raw(target.label()).width() + ": ".len())
}

pub fn theme_title(target: ThemeTarget, name: Option<&str>, width: u16) -> String {
    let label = target.label();
    match name {
        Some(name) => format!(
            "{label}: {}",
            theme_label(name, theme_name_width(target, width))
        ),
        None => label.to_owned(),
    }
}

pub fn comparison_titles(names: [Option<&str>; 2], widths: [u16; 2]) -> [String; 2] {
    const MIN_NAME_WIDTH: usize = 10;
    let targets = [ThemeTarget::Pin, ThemeTarget::Current];
    let show_names = targets
        .into_iter()
        .zip(widths)
        .all(|(target, width)| theme_name_width(target, width) >= MIN_NAME_WIDTH);
    let names = if show_names { names } else { [None; 2] };
    std::array::from_fn(|index| theme_title(targets[index], names[index], widths[index]))
}

pub fn panel(title: impl Into<String>, theme: UiTheme) -> Block<'static> {
    panel_with_title(
        Line::styled(title.into().trim().to_owned(), theme.heading_style()),
        theme,
    )
}

pub fn panel_with_title(title: Line<'static>, theme: UiTheme) -> Block<'static> {
    Block::default()
        .padding(Padding::new(0, 0, 1, 0))
        .title(title)
        .style(theme.base_style())
}

pub fn draw_modal(frame: &mut Frame, layout: &ModalLayout, title: &str, theme: UiTheme) {
    frame.render_widget(Clear, layout.outer);
    frame.render_widget(
        Block::bordered()
            .border_type(BorderType::Plain)
            .border_style(theme.base_style())
            .style(theme.base_style()),
        layout.outer,
    );
    frame.render_widget(
        Paragraph::new(title).style(theme.heading_style()),
        layout.title,
    );
}

pub fn draw_input(frame: &mut Frame, input: &Input, area: Rect, style: Style, focused: bool) {
    if area.is_empty() {
        return;
    }
    let scroll = input.visual_scroll(usize::from(area.width - 1));
    frame.render_widget(
        Paragraph::new(Line::styled(input.value(), style)).scroll((0, scroll as u16)),
        area,
    );
    if focused {
        let cursor = input.visual_cursor().saturating_sub(scroll) as u16;
        frame.set_cursor_position((area.x + cursor, area.y));
    }
}

pub fn wrap_text(value: &str, width: u16, max_rows: usize) -> Vec<Line<'static>> {
    wrap_line(Line::raw(value.to_owned()), width, max_rows)
}

pub fn wrap_line(line: Line<'static>, width: u16, max_rows: usize) -> Vec<Line<'static>> {
    if width == 0 || max_rows == 0 {
        return Vec::new();
    }
    let graphemes: Vec<_> = line.styled_graphemes(Style::default()).collect();
    let mut offset = 0;
    let mut rows = Vec::new();
    while rows.len() + 1 < max_rows && offset < graphemes.len() {
        let mut row = Vec::new();
        let mut used = 0;
        while let Some(grapheme) = graphemes.get(offset) {
            let columns = Span::raw(grapheme.symbol).width();
            if used + columns > usize::from(width) {
                break;
            }
            used += columns;
            row.push(Span::styled(grapheme.symbol.to_owned(), grapheme.style));
            offset += 1;
        }
        rows.push(Line::from(row));
    }
    if offset < graphemes.len() || rows.is_empty() {
        let clipped = graphemes[offset..]
            .iter()
            .map(|g| Span::raw(g.symbol).width())
            .sum::<usize>()
            > usize::from(width);
        let limit = if clipped {
            width.saturating_sub(2)
        } else {
            width
        };
        let mut row = Vec::new();
        let mut used = 0;
        for grapheme in &graphemes[offset..] {
            let columns = Span::raw(grapheme.symbol).width();
            if used + columns > usize::from(limit) {
                break;
            }
            row.push(Span::styled(grapheme.symbol.to_owned(), grapheme.style));
            used += columns;
        }
        if clipped {
            row.push(Span::raw(".".repeat(usize::from(width.min(2)))));
        }
        rows.push(Line::from(row));
    }
    rows
}

/// Prepare the viewport for single-line items, preserving any trailing empty rows.
pub fn prepare_list(state: &mut ListState, length: usize, rows: usize) {
    if rows == 0 {
        return;
    }
    let Some(last) = length.checked_sub(1) else {
        state.select(None);
        return;
    };
    if state.selected().is_some_and(|selected| selected > last) {
        state.select(Some(last));
    }
    let mut offset = state.offset().min(last);
    if let Some(selected) = state.selected() {
        if selected < offset {
            offset = selected;
        } else if selected >= offset + rows {
            offset = selected + 1 - rows;
        }
    }
    *state.offset_mut() = offset;
}

pub fn draw_list(frame: &mut Frame, list: List<'_>, area: Rect, state: &ListState) {
    // Ratatui receives a local copy; persistent viewport changes are prepared before drawing.
    let mut state = *state;
    frame.render_stateful_widget(list, area, &mut state);
}

/// Draw a viewport thumb in the column immediately after the scrolling content.
pub fn draw_scrollbar(
    frame: &mut Frame,
    content: Rect,
    length: usize,
    offset: usize,
    theme: UiTheme,
) {
    let rows = usize::from(content.height);
    let visible = length.saturating_sub(offset).min(rows);
    if content.width == 0 || visible == 0 || visible == length {
        return;
    }
    let thumb = rows.saturating_mul(visible) / length;
    let thumb = thumb.max(1);
    let start = (rows - thumb).saturating_mul(offset) / (length - visible);
    for row in start..start + thumb {
        if let Some(cell) = frame
            .buffer_mut()
            .cell_mut((content.right(), content.y + row as u16))
        {
            cell.set_symbol("┃").set_style(theme.base_style());
        }
    }
}
