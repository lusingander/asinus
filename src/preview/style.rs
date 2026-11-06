use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::theme::Theme;

pub fn bold(color: Color) -> Style {
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

pub fn prompt(theme: &Theme, command: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled("~/project", Color::from(theme.palette.bright[4])),
        Span::styled(" $ ", Color::from(theme.palette.normal[2])),
        Span::raw(command),
    ])
}
