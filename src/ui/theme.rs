use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Default, Clone, Copy)]
pub struct UiStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    /// An explicit empty set clears the default modifiers for the role.
    pub modifiers: Option<Modifier>,
}

impl UiStyle {
    fn foreground(color: Color) -> Self {
        Self {
            fg: Some(color),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiTheme {
    pub fg: Color,
    pub bg: Color,
    pub heading: UiStyle,
    pub selection: UiStyle,
    pub focus: UiStyle,
    pub key: UiStyle,
    pub condition_key: UiStyle,
    pub muted: UiStyle,
    pub search_match: UiStyle,
    pub comparison_diff: UiStyle,
    pub notice: UiStyle,
    pub warning: UiStyle,
    pub error: UiStyle,
    pub success: UiStyle,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            heading: UiStyle::default(),
            selection: UiStyle::default(),
            focus: UiStyle::foreground(Color::Cyan),
            key: UiStyle::foreground(Color::Cyan),
            condition_key: UiStyle::foreground(Color::Green),
            muted: UiStyle::default(),
            search_match: UiStyle::foreground(Color::Yellow),
            comparison_diff: UiStyle::foreground(Color::Yellow),
            notice: UiStyle::foreground(Color::Blue),
            warning: UiStyle::foreground(Color::Yellow),
            error: UiStyle::foreground(Color::Red),
            success: UiStyle::foreground(Color::Green),
        }
    }
}

impl UiTheme {
    pub fn base_style(self) -> Style {
        Style::reset().fg(self.fg).bg(self.bg)
    }

    pub fn role_style(self, role: UiStyle) -> Style {
        self.resolve_style(role, Modifier::empty())
    }

    pub fn heading_style(self) -> Style {
        self.resolve_style(self.heading, Modifier::BOLD)
    }

    pub fn focus_style(self) -> Style {
        self.resolve_style(self.focus, Modifier::BOLD)
    }

    pub fn selection_style(self) -> Style {
        self.resolve_style(self.selection, Modifier::REVERSED)
    }

    pub fn help_key_style(self) -> Style {
        self.resolve_style(self.key, Modifier::BOLD)
    }

    pub fn search_match_style(self) -> Style {
        self.resolve_style(self.search_match, Modifier::BOLD)
    }

    pub fn search_error_style(self) -> Style {
        self.resolve_style(self.error, Modifier::BOLD)
    }

    fn resolve_style(self, role: UiStyle, default_modifiers: Modifier) -> Style {
        self.base_style()
            .fg(role.fg.unwrap_or(self.fg))
            .bg(role.bg.unwrap_or(self.bg))
            .add_modifier(role.modifiers.unwrap_or(default_modifiers))
    }
}
