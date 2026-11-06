use crate::{app::App, theme::Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeTarget {
    Current,
    Pin,
}

impl ThemeTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Current => "Current",
            Self::Pin => "Pin",
        }
    }

    pub fn other(self) -> Self {
        match self {
            Self::Current => Self::Pin,
            Self::Pin => Self::Current,
        }
    }
}

impl App {
    pub fn theme_for_target(&self, target: ThemeTarget) -> Option<&Theme> {
        match target {
            ThemeTarget::Current => self.theme(),
            ThemeTarget::Pin => self.pinned_theme(),
        }
    }

    pub fn default_theme_target(&self) -> Option<ThemeTarget> {
        if self.theme().is_some() {
            Some(ThemeTarget::Current)
        } else if self.pinned_theme().is_some() {
            Some(ThemeTarget::Pin)
        } else {
            None
        }
    }

    pub fn can_switch_theme_target(&self) -> bool {
        self.theme().is_some() && self.pinned_theme().is_some()
    }
}
