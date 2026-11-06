mod catalog;

use std::fmt;

use ratatui::style::Color;

pub use crate::theme::catalog::THEMES;
use crate::{color_math, filter_criteria::ThemeMetrics};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(u8, u8, u8);

impl Rgb {
    /// WCAG 2.2 contrast ratio for two opaque sRGB colors.
    pub fn contrast(self, other: Self) -> f64 {
        color_math::contrast([self.0, self.1, self.2], [other.0, other.1, other.2])
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

impl From<Rgb> for Color {
    fn from(Rgb(red, green, blue): Rgb) -> Self {
        Self::Rgb(red, green, blue)
    }
}

#[derive(Clone)]
pub struct Theme {
    pub name: &'static str,
    pub foreground: Rgb,
    pub background: Rgb,
    pub cursor: Option<Rgb>,
    pub selection: Option<Rgb>,
    pub meta: Metadata,
    pub palette: Palette,
    pub metrics: ThemeMetrics,
}

#[derive(Clone)]
pub struct Metadata {
    pub is_dark: bool,
    pub credits: &'static [Credit],
}

#[derive(Debug, PartialEq, Eq)]
pub struct Credit {
    pub name: &'static str,
    pub link: &'static str,
}

#[derive(Clone)]
pub struct Palette {
    pub normal: [Rgb; 8],
    pub bright: [Rgb; 8],
}
