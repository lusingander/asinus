/// Unrounded metrics validated by the theme generator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeMetrics {
    /// OKLCH lightness scaled to 0..=100.
    pub background_lightness: f64,
    /// OKLCH chroma in its original scale.
    pub background_chroma: f64,
    /// OKLCH hue in 0..360 degrees; absent for exact sRGB grays.
    pub background_hue: Option<f64>,
    /// WCAG 2.2 contrast ratio for the normal foreground and background.
    pub text_contrast: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Brightness {
    VeryDark,
    Dark,
    Medium,
    Light,
    VeryLight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundColor {
    Gray,
    RedPink,
    OrangeBrown,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextContrast {
    Low,
    Medium,
    High,
}

impl Brightness {
    pub const ALL: [Self; 5] = [
        Self::VeryDark,
        Self::Dark,
        Self::Medium,
        Self::Light,
        Self::VeryLight,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::VeryDark => "very-dark",
            Self::Dark => "dark",
            Self::Medium => "medium",
            Self::Light => "light",
            Self::VeryLight => "very-light",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::VeryDark => "Very dark",
            Self::Dark => "Dark",
            Self::Medium => "Medium",
            Self::Light => "Light",
            Self::VeryLight => "Very light",
        }
    }

    pub fn from_id(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|choice| choice.id() == value)
    }
}

impl BackgroundColor {
    pub const ALL: [Self; 8] = [
        Self::Gray,
        Self::RedPink,
        Self::OrangeBrown,
        Self::Yellow,
        Self::Green,
        Self::Cyan,
        Self::Blue,
        Self::Purple,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Gray => "gray",
            Self::RedPink => "red-pink",
            Self::OrangeBrown => "orange-brown",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::Cyan => "cyan",
            Self::Blue => "blue",
            Self::Purple => "purple",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Gray => "Gray",
            Self::RedPink => "Red/Pink",
            Self::OrangeBrown => "Orange/Brown",
            Self::Yellow => "Yellow",
            Self::Green => "Green",
            Self::Cyan => "Cyan",
            Self::Blue => "Blue",
            Self::Purple => "Purple",
        }
    }

    pub fn from_id(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|choice| choice.id() == value)
    }
}

impl TextContrast {
    pub const ALL: [Self; 3] = [Self::Low, Self::Medium, Self::High];

    pub fn id(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }

    pub fn from_id(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|choice| choice.id() == value)
    }
}

impl ThemeMetrics {
    pub fn brightness(&self) -> Brightness {
        match self.background_lightness {
            value if value < 25.0 => Brightness::VeryDark,
            value if value < 40.0 => Brightness::Dark,
            value if value < 70.0 => Brightness::Medium,
            value if value < 95.0 => Brightness::Light,
            _ => Brightness::VeryLight,
        }
    }

    pub fn background_color(&self) -> BackgroundColor {
        if self.background_chroma <= 0.015 {
            return BackgroundColor::Gray;
        }
        match self.background_hue.expect("chromatic background has a hue") {
            hue if hue < 40.0 => BackgroundColor::RedPink,
            hue if hue < 85.0 => BackgroundColor::OrangeBrown,
            hue if hue < 115.0 => BackgroundColor::Yellow,
            hue if hue < 170.0 => BackgroundColor::Green,
            hue if hue < 230.0 => BackgroundColor::Cyan,
            hue if hue < 285.0 => BackgroundColor::Blue,
            hue if hue < 345.0 => BackgroundColor::Purple,
            _ => BackgroundColor::RedPink,
        }
    }

    pub fn contrast(&self) -> TextContrast {
        match self.text_contrast {
            value if value < 7.0 => TextContrast::Low,
            value if value < 12.0 => TextContrast::Medium,
            _ => TextContrast::High,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::theme::THEMES;

    fn sample() -> ThemeMetrics {
        ThemeMetrics {
            background_lightness: 50.0,
            background_chroma: 0.02,
            background_hue: Some(250.0),
            text_contrast: 10.0,
        }
    }

    #[rstest]
    #[case(0.0, Brightness::VeryDark)]
    #[case(25.0_f64.next_down(), Brightness::VeryDark)]
    #[case(25.0, Brightness::Dark)]
    #[case(40.0_f64.next_down(), Brightness::Dark)]
    #[case(40.0, Brightness::Medium)]
    #[case(70.0_f64.next_down(), Brightness::Medium)]
    #[case(70.0, Brightness::Light)]
    #[case(95.0_f64.next_down(), Brightness::Light)]
    #[case(95.0, Brightness::VeryLight)]
    #[case(100.0, Brightness::VeryLight)]
    fn brightness_includes_each_lower_boundary(
        #[case] background_lightness: f64,
        #[case] expected: Brightness,
    ) {
        let metrics = ThemeMetrics {
            background_lightness,
            ..sample()
        };
        assert_eq!(metrics.brightness(), expected);
    }

    #[rstest]
    #[case(0.0, None, BackgroundColor::Gray)]
    #[case(0.005, Some(250.0), BackgroundColor::Gray)]
    #[case(0.015, Some(250.0), BackgroundColor::Gray)]
    #[case(0.015_f64.next_up(), Some(250.0), BackgroundColor::Blue)]
    fn gray_includes_the_chroma_boundary_before_hue_is_used(
        #[case] background_chroma: f64,
        #[case] background_hue: Option<f64>,
        #[case] expected: BackgroundColor,
    ) {
        let metrics = ThemeMetrics {
            background_chroma,
            background_hue,
            ..sample()
        };
        assert_eq!(metrics.background_color(), expected);
    }

    #[rstest]
    #[case(0.0, BackgroundColor::RedPink)]
    #[case(40.0_f64.next_down(), BackgroundColor::RedPink)]
    #[case(40.0, BackgroundColor::OrangeBrown)]
    #[case(85.0_f64.next_down(), BackgroundColor::OrangeBrown)]
    #[case(85.0, BackgroundColor::Yellow)]
    #[case(115.0_f64.next_down(), BackgroundColor::Yellow)]
    #[case(115.0, BackgroundColor::Green)]
    #[case(170.0_f64.next_down(), BackgroundColor::Green)]
    #[case(170.0, BackgroundColor::Cyan)]
    #[case(230.0_f64.next_down(), BackgroundColor::Cyan)]
    #[case(230.0, BackgroundColor::Blue)]
    #[case(285.0_f64.next_down(), BackgroundColor::Blue)]
    #[case(285.0, BackgroundColor::Purple)]
    #[case(345.0_f64.next_down(), BackgroundColor::Purple)]
    #[case(345.0, BackgroundColor::RedPink)]
    #[case(360.0_f64.next_down(), BackgroundColor::RedPink)]
    fn hue_boundaries_include_the_lower_edge_and_red_crosses_zero(
        #[case] hue: f64,
        #[case] expected: BackgroundColor,
    ) {
        let metrics = ThemeMetrics {
            background_hue: Some(hue),
            ..sample()
        };
        assert_eq!(metrics.background_color(), expected);
    }

    #[rstest]
    #[case(1.0, TextContrast::Low)]
    #[case(7.0_f64.next_down(), TextContrast::Low)]
    #[case(7.0, TextContrast::Medium)]
    #[case(12.0_f64.next_down(), TextContrast::Medium)]
    #[case(12.0, TextContrast::High)]
    #[case(21.0, TextContrast::High)]
    fn contrast_includes_each_lower_boundary(
        #[case] text_contrast: f64,
        #[case] expected: TextContrast,
    ) {
        let metrics = ThemeMetrics {
            text_contrast,
            ..sample()
        };
        assert_eq!(metrics.contrast(), expected);
    }

    #[rstest]
    #[case("OneDark", BackgroundColor::Gray)]
    #[case("nord", BackgroundColor::Blue)]
    #[case("Catppuccin Mocha", BackgroundColor::Blue)]
    #[case("Rose Pine", BackgroundColor::Purple)]
    #[case("Tinacious Design (Dark)", BackgroundColor::Blue)]
    fn representative_backgrounds_use_unrounded_generated_metrics(
        #[case] name: &str,
        #[case] expected: BackgroundColor,
    ) {
        let theme = THEMES.iter().find(|theme| theme.name == name).unwrap();
        assert_eq!(theme.metrics.background_color(), expected);
    }
}
