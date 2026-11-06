use crate::theme::{Rgb, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRole {
    Foreground,
    Background,
    Normal(usize),
    Bright(usize),
    Cursor,
    Selection,
}

impl ColorRole {
    pub const ALL: [Self; 20] = [
        Self::Foreground,
        Self::Background,
        Self::Normal(0),
        Self::Normal(1),
        Self::Normal(2),
        Self::Normal(3),
        Self::Normal(4),
        Self::Normal(5),
        Self::Normal(6),
        Self::Normal(7),
        Self::Bright(0),
        Self::Bright(1),
        Self::Bright(2),
        Self::Bright(3),
        Self::Bright(4),
        Self::Bright(5),
        Self::Bright(6),
        Self::Bright(7),
        Self::Cursor,
        Self::Selection,
    ];

    pub fn label(self) -> String {
        let names = [
            "Black", "Red", "Green", "Yellow", "Blue", "Magenta", "Cyan", "White",
        ];
        match self {
            Self::Foreground => "Foreground".into(),
            Self::Background => "Background".into(),
            Self::Normal(index) => names[index].into(),
            Self::Bright(index) => format!("Bright {}", names[index]),
            Self::Cursor => "Cursor".into(),
            Self::Selection => "Selection".into(),
        }
    }

    pub fn value(self, theme: &Theme) -> Option<Rgb> {
        match self {
            Self::Foreground => Some(theme.foreground),
            Self::Background => Some(theme.background),
            Self::Normal(index) => Some(theme.palette.normal[index]),
            Self::Bright(index) => Some(theme.palette.bright[index]),
            Self::Cursor => theme.cursor,
            Self::Selection => theme.selection,
        }
    }

    pub fn contrast(self, theme: &Theme) -> Option<f64> {
        match self {
            Self::Foreground => Some(theme.metrics.text_contrast),
            Self::Normal(_) | Self::Bright(_) => {
                Some(self.value(theme)?.contrast(theme.background))
            }
            _ => None,
        }
    }

    pub fn relation(self, a: &Theme, b: &Theme) -> ColorRelation {
        match (self.value(a), self.value(b)) {
            (Some(a), Some(b)) if a == b => ColorRelation::Same,
            (Some(_), Some(_)) => ColorRelation::Different,
            _ => ColorRelation::Unspecified,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRelation {
    Same,
    Different,
    Unspecified,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme;

    #[test]
    fn equal_ansi_values_can_have_different_background_contrast() {
        let themes = theme::THEMES;
        let a = themes
            .iter()
            .find(|theme| theme.name == "3024 Day")
            .unwrap();
        let b = themes
            .iter()
            .find(|theme| theme.name == "3024 Night")
            .unwrap();
        let same = ColorRole::ALL
            .iter()
            .filter(|role| role.relation(a, b) == ColorRelation::Same)
            .count();
        assert_eq!(same, 16);
        let role = ColorRole::Bright(0);
        assert_eq!(role.value(a).unwrap().to_string(), "#5c5855");
        assert!(role.contrast(a).unwrap() > 6.5);
        assert!(role.contrast(b).unwrap() < 3.0);
    }

    #[test]
    fn unspecified_optional_colors_are_never_equal_or_evaluated() {
        let themes = theme::THEMES;
        let a = themes
            .iter()
            .find(|theme| theme.name == "DraculaPlus")
            .unwrap();
        let b = themes.iter().find(|theme| theme.name == "Dracula").unwrap();
        for role in [ColorRole::Cursor, ColorRole::Selection] {
            assert_eq!(role.relation(a, a), ColorRelation::Unspecified);
            assert_eq!(role.relation(a, b), ColorRelation::Unspecified);
            assert!(role.contrast(b).is_none());
        }
        assert!(ColorRole::Background.contrast(a).is_none());
        assert_eq!(ColorRole::Foreground.relation(a, b), ColorRelation::Same);
    }
}
