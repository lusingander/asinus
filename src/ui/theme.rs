use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy)]
pub struct UiTheme {
    pub fg: Color,
    pub bg: Color,
    pub heading: Color,
    pub selection_fg: Color,
    pub selection_bg: Color,
    pub focus: Color,
    pub key: Color,
    pub condition_key: Color,
    pub muted: Color,
    pub match_fg: Color,
    pub comparison_diff: Color,
    pub notice: Color,
    pub warning: Color,
    pub error: Color,
    pub success: Color,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            heading: Color::Reset,
            selection_fg: Color::Reset,
            selection_bg: Color::Reset,
            focus: Color::Cyan,
            key: Color::Cyan,
            condition_key: Color::Green,
            muted: Color::Reset,
            match_fg: Color::Yellow,
            comparison_diff: Color::Yellow,
            notice: Color::Blue,
            warning: Color::Yellow,
            error: Color::Red,
            success: Color::Green,
        }
    }
}

impl UiTheme {
    pub fn condition_key_style(self) -> Style {
        self.base_style().fg(self.condition_key)
    }

    pub fn focus_style(self) -> Style {
        self.base_style()
            .fg(self.focus)
            .add_modifier(Modifier::BOLD)
    }

    pub fn key_style(self) -> Style {
        Style::default().fg(self.key)
    }

    pub fn help_key_style(self) -> Style {
        self.base_style().fg(self.key).add_modifier(Modifier::BOLD)
    }

    pub fn base_style(self) -> Style {
        Style::reset().fg(self.fg).bg(self.bg)
    }

    pub fn heading_style(self) -> Style {
        self.base_style()
            .fg(self.heading)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selection_style(self) -> Style {
        self.base_style()
            .fg(self.selection_fg)
            .bg(self.selection_bg)
            .add_modifier(Modifier::REVERSED)
    }

    pub fn match_style(self) -> Style {
        self.base_style()
            .fg(self.match_fg)
            .add_modifier(Modifier::BOLD)
    }
}
