use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyFormat {
    Name,
    Json,
}

impl CopyFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Json => "JSON",
        }
    }

    pub fn text(self, theme: &Theme) -> String {
        if self == Self::Name {
            return theme.name.to_owned();
        }
        serde_json::to_string_pretty(&JsonColors(theme))
            .expect("RGB strings have a JSON representation")
    }
}

struct JsonColors<'a>(&'a Theme);

impl Serialize for JsonColors<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let theme = self.0;
        let mut colors = serializer.serialize_map(None)?;
        colors.serialize_entry("foreground", &theme.foreground.to_string())?;
        colors.serialize_entry("background", &theme.background.to_string())?;
        for (names, palette) in [
            (
                [
                    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
                ],
                &theme.palette.normal,
            ),
            (
                [
                    "brightBlack",
                    "brightRed",
                    "brightGreen",
                    "brightYellow",
                    "brightBlue",
                    "brightMagenta",
                    "brightCyan",
                    "brightWhite",
                ],
                &theme.palette.bright,
            ),
        ] {
            for (name, color) in names.into_iter().zip(palette) {
                colors.serialize_entry(name, &color.to_string())?;
            }
        }
        for (name, color) in [("cursor", theme.cursor), ("selection", theme.selection)] {
            if let Some(color) = color {
                colors.serialize_entry(name, &color.to_string())?;
            }
        }
        colors.end()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::theme;

    #[rstest]
    #[case(true, true, &["cursor", "selection"])]
    #[case(true, false, &["cursor"])]
    #[case(false, true, &["selection"])]
    #[case(false, false, &[])]
    fn copies_json_colors_in_semantic_order(
        #[case] include_cursor: bool,
        #[case] include_selection: bool,
        #[case] optional_keys: &[&str],
    ) {
        let mut theme = theme::THEMES
            .iter()
            .find(|theme| theme.name == "3024 Day")
            .unwrap()
            .clone();
        theme.cursor = theme.cursor.filter(|_| include_cursor);
        theme.selection = theme.selection.filter(|_| include_selection);

        let text = CopyFormat::Json.text(&theme);
        let actual: Vec<_> = text
            .lines()
            .filter_map(|line| {
                line.strip_prefix("  \"")?
                    .split_once("\": ")
                    .map(|(key, _)| key)
            })
            .collect();
        let expected: Vec<_> = [
            "foreground",
            "background",
            "black",
            "red",
            "green",
            "yellow",
            "blue",
            "magenta",
            "cyan",
            "white",
            "brightBlack",
            "brightRed",
            "brightGreen",
            "brightYellow",
            "brightBlue",
            "brightMagenta",
            "brightCyan",
            "brightWhite",
        ]
        .into_iter()
        .chain(optional_keys.iter().copied())
        .collect();
        assert_eq!(actual, expected);
    }
}
