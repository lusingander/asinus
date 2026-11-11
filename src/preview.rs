mod code;
pub mod custom;
mod git;
mod logs;
mod palette;
mod shell;
mod style;
mod tui;

use ratatui::text::Line;
use serde::{Deserialize, Serialize};

use crate::theme::Theme;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    #[default]
    Palette,
    Shell,
    Git,
    Logs,
    Tui,
    Code,
}

impl Preset {
    pub fn name(self) -> &'static str {
        match self {
            Self::Palette => "Palette",
            Self::Shell => "Shell",
            Self::Git => "Git",
            Self::Logs => "Logs",
            Self::Tui => "TUI",
            Self::Code => "Code",
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Palette => Self::Code,
            Self::Shell => Self::Palette,
            Self::Git => Self::Shell,
            Self::Logs => Self::Git,
            Self::Tui => Self::Logs,
            Self::Code => Self::Tui,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Palette => Self::Shell,
            Self::Shell => Self::Git,
            Self::Git => Self::Logs,
            Self::Logs => Self::Tui,
            Self::Tui => Self::Code,
            Self::Code => Self::Palette,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleSize {
    Small,
    Medium,
    Large,
}

impl SampleSize {
    pub const ALL: [Self; 3] = [Self::Small, Self::Medium, Self::Large];

    pub fn dimensions(self) -> (u16, u16) {
        match self {
            Self::Small => (40, 12),
            Self::Medium => (52, 18),
            Self::Large => (70, 24),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentSize {
    #[default]
    Auto,
    Small,
    Medium,
    Large,
}

impl ContentSize {
    pub const ALL: [Self; 4] = [Self::Auto, Self::Small, Self::Medium, Self::Large];

    pub fn resolve(self, width: u16, height: u16) -> SampleSize {
        match self {
            Self::Small => SampleSize::Small,
            Self::Medium => SampleSize::Medium,
            Self::Large => SampleSize::Large,
            Self::Auto => SampleSize::ALL
                .into_iter()
                .rev()
                .find(|size| {
                    let (columns, rows) = size.dimensions();
                    width >= columns && height >= rows
                })
                .unwrap_or(SampleSize::Small),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
        }
    }
}

pub fn lines(theme: &Theme, preset: Preset, size: SampleSize) -> Vec<Line<'static>> {
    match preset {
        Preset::Palette => palette::lines(theme, size),
        Preset::Shell => shell::lines(theme, size),
        Preset::Git => git::lines(theme, size),
        Preset::Logs => logs::lines(theme, size),
        Preset::Tui => tui::lines(theme, size),
        Preset::Code => code::lines(theme, size),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(0, 0, SampleSize::Small)]
    #[case(39, 12, SampleSize::Small)]
    #[case(40, 11, SampleSize::Small)]
    #[case(40, 12, SampleSize::Small)]
    #[case(51, 18, SampleSize::Small)]
    #[case(52, 17, SampleSize::Small)]
    #[case(52, 18, SampleSize::Medium)]
    #[case(69, 24, SampleSize::Medium)]
    #[case(70, 23, SampleSize::Medium)]
    #[case(70, 24, SampleSize::Large)]
    #[case(120, 3, SampleSize::Small)]
    #[case(20, 100, SampleSize::Small)]
    fn auto_requires_both_reference_dimensions(
        #[case] width: u16,
        #[case] height: u16,
        #[case] expected: SampleSize,
    ) {
        assert_eq!(ContentSize::Auto.resolve(width, height), expected);
    }

    #[rstest]
    fn fixed_content_survives_resizing(
        #[values(ContentSize::Small, ContentSize::Medium, ContentSize::Large)] content: ContentSize,
    ) {
        assert_eq!(content.resolve(1, 1), content.resolve(160, 100));
    }

    #[rstest]
    fn samples_fit_their_shared_reference_and_keep_the_same_text_across_themes(
        #[values(SampleSize::Small, SampleSize::Medium, SampleSize::Large)] size: SampleSize,
    ) {
        let themes = crate::theme::THEMES;
        let (width, height) = size.dimensions();
        for preset in [
            Preset::Palette,
            Preset::Shell,
            Preset::Git,
            Preset::Logs,
            Preset::Tui,
            Preset::Code,
        ] {
            let reference = lines(&themes[0], preset, size);
            assert!(
                reference.len() <= usize::from(height),
                "{preset:?} {size:?}"
            );
            assert!(
                reference
                    .iter()
                    .all(|line| line.width() <= usize::from(width)),
                "{preset:?} {size:?}"
            );
            if preset != Preset::Palette {
                let text = |theme| {
                    lines(theme, preset, size)
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                };
                for theme in themes {
                    assert_eq!(text(theme), text(&themes[0]));
                }
            }
        }
    }
}
