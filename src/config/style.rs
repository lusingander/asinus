use ratatui::style::{Color, Modifier};
use serde::{Deserialize, Deserializer, de};

use crate::ui::theme::{UiStyle, UiTheme};

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StyleConfig {
    base: BaseColors,
    heading: StyleOverride,
    focus: StyleOverride,
    selection: StyleOverride,
    key: StyleOverride,
    muted: StyleOverride,
    condition_key: StyleOverride,
    search_match: StyleOverride,
    comparison_diff: StyleOverride,
    notice: StyleOverride,
    success: StyleOverride,
    warning: StyleOverride,
    error: StyleOverride,
}

impl StyleConfig {
    pub fn resolve(self) -> UiTheme {
        let defaults = UiTheme::default();
        UiTheme {
            fg: self.base.fg.map_or(defaults.fg, |color| color.0),
            bg: self.base.bg.map_or(defaults.bg, |color| color.0),
            heading: self.heading.resolve(defaults.heading),
            focus: self.focus.resolve(defaults.focus),
            selection: self.selection.resolve(defaults.selection),
            key: self.key.resolve(defaults.key),
            muted: self.muted.resolve(defaults.muted),
            condition_key: self.condition_key.resolve(defaults.condition_key),
            search_match: self.search_match.resolve(defaults.search_match),
            comparison_diff: self.comparison_diff.resolve(defaults.comparison_diff),
            notice: self.notice.resolve(defaults.notice),
            success: self.success.resolve(defaults.success),
            warning: self.warning.resolve(defaults.warning),
            error: self.error.resolve(defaults.error),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct BaseColors {
    fg: Option<ConfigColor>,
    bg: Option<ConfigColor>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct StyleOverride {
    fg: Option<ConfigColor>,
    bg: Option<ConfigColor>,
    modifiers: Option<Vec<ConfigModifier>>,
}

impl StyleOverride {
    fn resolve(self, defaults: UiStyle) -> UiStyle {
        UiStyle {
            fg: self.fg.map(|color| color.0).or(defaults.fg),
            bg: self.bg.map(|color| color.0).or(defaults.bg),
            modifiers: self
                .modifiers
                .map(|modifiers| modifiers.into_iter().map(|modifier| modifier.0).collect())
                .or(defaults.modifiers),
        }
    }
}

#[derive(Debug)]
struct ConfigColor(Color);

impl<'de> Deserialize<'de> for ConfigColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        let color = match value.as_str() {
            "default" => Color::Reset,
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::Gray,
            "bright_black" => Color::DarkGray,
            "bright_red" => Color::LightRed,
            "bright_green" => Color::LightGreen,
            "bright_yellow" => Color::LightYellow,
            "bright_blue" => Color::LightBlue,
            "bright_magenta" => Color::LightMagenta,
            "bright_cyan" => Color::LightCyan,
            "bright_white" => Color::White,
            hex if hex.starts_with('#')
                && hex.len() == 7
                && hex.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit) =>
            {
                hex.parse().map_err(de::Error::custom)?
            }
            _ => {
                return Err(de::Error::custom(format!(
                    "invalid color {value:?}, expected an ANSI color name, #RRGGBB, or default"
                )));
            }
        };
        Ok(Self(color))
    }
}

#[derive(Debug)]
struct ConfigModifier(Modifier);

impl<'de> Deserialize<'de> for ConfigModifier {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        let modifier = match value.as_str() {
            "bold" => Modifier::BOLD,
            "dim" => Modifier::DIM,
            "underline" => Modifier::UNDERLINED,
            "reverse" => Modifier::REVERSED,
            _ => {
                return Err(de::Error::unknown_variant(
                    &value,
                    &["bold", "dim", "underline", "reverse"],
                ));
            }
        };
        Ok(Self(modifier))
    }
}
