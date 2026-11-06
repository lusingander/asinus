use std::ops::Range;

use crate::{
    filter_criteria::{BackgroundColor, Brightness, TextContrast},
    marks::Mark,
    theme::Theme,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
    #[default]
    All,
    Dark,
    Light,
}

impl Appearance {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::All => Self::Light,
            Self::Dark => Self::All,
            Self::Light => Self::Dark,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Dark,
            Self::Dark => Self::Light,
            Self::Light => Self::All,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Favorite {
    #[default]
    All,
    Only,
    Exclude,
}

impl Favorite {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Only => "Only",
            Self::Exclude => "Exclude",
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::All => Self::Exclude,
            Self::Only => Self::All,
            Self::Exclude => Self::Only,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Only,
            Self::Only => Self::Exclude,
            Self::Exclude => Self::All,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Ignored {
    #[default]
    Hide,
    Show,
    Only,
}

impl Ignored {
    pub fn label(self) -> &'static str {
        match self {
            Self::Hide => "Hide",
            Self::Show => "Show",
            Self::Only => "Only",
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Hide => Self::Only,
            Self::Show => Self::Hide,
            Self::Only => Self::Show,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Hide => Self::Show,
            Self::Show => Self::Only,
            Self::Only => Self::Hide,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Filter {
    terms: Vec<String>,
    pub appearance: Appearance,
    pub favorite: Favorite,
    pub ignored: Ignored,
    pub brightness: Option<Brightness>,
    pub background_color: Option<BackgroundColor>,
    pub text_contrast: Option<TextContrast>,
}

impl Filter {
    pub fn parse(query: &str) -> Result<Self, String> {
        let mut filter = Self::default();
        let mut seen = Vec::new();
        for (word, quoted, _) in tokens(query)? {
            if !quoted && let Some((key, value)) = word.split_once(':') {
                if seen.contains(&key.to_owned()) {
                    return Err(format!("Use only one {key} filter"));
                }
                match key {
                    "type" => {
                        filter.appearance = match value {
                            "dark" => Appearance::Dark,
                            "light" => Appearance::Light,
                            _ => return Err("type: choose dark or light".into()),
                        }
                    }
                    "favorite" => {
                        filter.favorite = match value {
                            "true" => Favorite::Only,
                            "false" => Favorite::Exclude,
                            _ => return Err("favorite: choose true or false".into()),
                        }
                    }
                    "ignore" => {
                        filter.ignored = match value {
                            "false" => Ignored::Hide,
                            "all" => Ignored::Show,
                            "true" => Ignored::Only,
                            _ => return Err("ignore: choose false, all or true".into()),
                        }
                    }
                    "brightness" => {
                        filter.brightness = Some(
                            Brightness::from_id(value)
                                .ok_or("brightness: invalid value; see Help")?,
                        );
                    }
                    "bg" => {
                        filter.background_color = Some(
                            BackgroundColor::from_id(value).ok_or("bg: invalid value; see Help")?,
                        );
                    }
                    "contrast" => {
                        filter.text_contrast = Some(
                            TextContrast::from_id(value)
                                .ok_or("contrast: invalid value; see Help")?,
                        );
                    }
                    _ => {
                        return Err(format!("Unknown filter: see Help for keys. Key: {key}"));
                    }
                }
                seen.push(key.to_owned());
            } else {
                filter.terms.push(word);
            }
        }
        Ok(filter)
    }

    pub fn from_fields(
        name: &str,
        appearance: Appearance,
        favorite: Favorite,
        ignored: Ignored,
        brightness: Option<Brightness>,
        background_color: Option<BackgroundColor>,
        text_contrast: Option<TextContrast>,
    ) -> Result<Self, String> {
        let tokens = tokens(name)?;
        if tokens
            .iter()
            .any(|(word, quoted, _)| !quoted && word.contains(':'))
        {
            return Err("Use fields or Search".into());
        }
        Ok(Self {
            terms: tokens.into_iter().map(|(word, _, _)| word).collect(),
            appearance,
            favorite,
            ignored,
            brightness,
            background_color,
            text_contrast,
        })
    }

    pub fn name_query(&self) -> String {
        self.terms
            .iter()
            .map(|word| quote(word))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn condition_tokens(&self) -> Vec<String> {
        let mut tokens = Vec::new();
        match self.appearance {
            Appearance::All => {}
            Appearance::Dark => tokens.push("type:dark".into()),
            Appearance::Light => tokens.push("type:light".into()),
        }
        match self.favorite {
            Favorite::All => {}
            Favorite::Only => tokens.push("favorite:true".into()),
            Favorite::Exclude => tokens.push("favorite:false".into()),
        }
        tokens.push(
            match self.ignored {
                Ignored::Hide => "ignore:false",
                Ignored::Show => "ignore:all",
                Ignored::Only => "ignore:true",
            }
            .into(),
        );
        if let Some(value) = self.brightness {
            tokens.push(format!("brightness:{}", value.id()));
        }
        if let Some(value) = self.background_color {
            tokens.push(format!("bg:{}", value.id()));
        }
        if let Some(value) = self.text_contrast {
            tokens.push(format!("contrast:{}", value.id()));
        }
        tokens
    }

    pub fn query(&self) -> String {
        let name = self.name_query();
        let suffix = self.condition_tokens().join(" ");
        if name.is_empty() {
            suffix
        } else {
            format!("{name} {suffix}")
        }
    }

    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }

    pub fn matches(&self, theme: &Theme, mark: Option<Mark>) -> bool {
        let appearance = match self.appearance {
            Appearance::All => true,
            Appearance::Dark => theme.meta.is_dark,
            Appearance::Light => !theme.meta.is_dark,
        };
        let favorite = match self.favorite {
            Favorite::All => true,
            Favorite::Only => mark == Some(Mark::Favorite),
            Favorite::Exclude => mark != Some(Mark::Favorite),
        };
        let ignored = match self.ignored {
            Ignored::Hide => mark != Some(Mark::Ignored),
            Ignored::Show => true,
            Ignored::Only => mark == Some(Mark::Ignored),
        };
        appearance
            && favorite
            && ignored
            && self
                .brightness
                .is_none_or(|value| theme.metrics.brightness() == value)
            && self
                .background_color
                .is_none_or(|value| theme.metrics.background_color() == value)
            && self
                .text_contrast
                .is_none_or(|value| theme.metrics.contrast() == value)
            && self.name_matches(theme.name).is_some()
    }

    /// Match ranges use byte offsets in the original name, including after lowercase expansion.
    pub fn name_matches(&self, name: &str) -> Option<Vec<Range<usize>>> {
        let mut ranges = Vec::new();
        for term in &self.terms {
            let matches = if term.chars().any(char::is_uppercase) {
                occurrences(name, term)
            } else {
                let mut lowercase = String::new();
                let mut source = Vec::new();
                for (start, character) in name.char_indices() {
                    let lower_start = lowercase.len();
                    lowercase.extend(character.to_lowercase());
                    source.push((
                        lower_start..lowercase.len(),
                        start..start + character.len_utf8(),
                    ));
                }
                let needle: String = term.chars().flat_map(char::to_lowercase).collect();
                occurrences(&lowercase, &needle)
                    .into_iter()
                    .map(|range| {
                        let start = source
                            .iter()
                            .find(|(lower, _)| lower.contains(&range.start))
                            .unwrap()
                            .1
                            .start;
                        let end = source
                            .iter()
                            .find(|(lower, _)| lower.contains(&(range.end - 1)))
                            .unwrap()
                            .1
                            .end;
                        start..end
                    })
                    .collect()
            };
            if matches.is_empty() {
                return None;
            }
            ranges.extend(matches);
        }
        ranges.sort_by_key(|range| range.start);
        let mut merged: Vec<Range<usize>> = Vec::new();
        for range in ranges {
            if let Some(last) = merged.last_mut()
                && range.start <= last.end
            {
                last.end = last.end.max(range.end);
            } else {
                merged.push(range);
            }
        }
        Some(merged)
    }
}

fn occurrences(text: &str, needle: &str) -> Vec<Range<usize>> {
    text.char_indices()
        .filter_map(|(start, _)| {
            text[start..]
                .starts_with(needle)
                .then_some(start..start + needle.len())
        })
        .collect()
}

fn quote(word: &str) -> String {
    if word
        .chars()
        .any(|c| c.is_whitespace() || matches!(c, ':' | '"' | '\\'))
    {
        format!("\"{}\"", word.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        word.to_owned()
    }
}

/// Return byte ranges for known, unquoted condition keys, including the colon.
pub fn condition_key_ranges(query: &str) -> Vec<Range<usize>> {
    tokens(query)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(word, quoted, range)| {
            let (key, _) = word.split_once(':')?;
            (!quoted
                && matches!(
                    key,
                    "type" | "favorite" | "ignore" | "brightness" | "bg" | "contrast"
                ))
            .then_some(range.start..range.start + key.len() + 1)
        })
        .collect()
}

fn tokens(text: &str) -> Result<Vec<(String, bool, Range<usize>)>, String> {
    let mut chars = text.char_indices().peekable();
    let mut words = Vec::new();
    while let Some((start, character)) = chars.next() {
        if character.is_whitespace() {
            continue;
        }
        let quoted = character == '"';
        let mut word = String::new();
        if quoted {
            let mut closed = false;
            while let Some((_, character)) = chars.next() {
                match character {
                    '"' => {
                        closed = true;
                        break;
                    }
                    '\\' => match chars.next() {
                        Some((_, character @ ('"' | '\\'))) => word.push(character),
                        _ => return Err("Use \\\" or \\\\ inside quotes".into()),
                    },
                    _ => word.push(character),
                }
            }
            if !closed {
                return Err("Close the double quote".into());
            }
            if chars.peek().is_some_and(|(_, c)| !c.is_whitespace()) {
                return Err("Add a space after the quote".into());
            }
        } else {
            word.push(character);
            while let Some(&(_, character)) = chars.peek() {
                if character.is_whitespace() {
                    break;
                }
                if character == '"' {
                    return Err("Quote the entire name term".into());
                }
                word.push(chars.next().unwrap().1);
            }
        }
        if !word.is_empty() {
            let end = chars.peek().map_or(text.len(), |(index, _)| *index);
            words.push((word, quoted, start..end));
        }
    }
    Ok(words)
}

#[cfg(test)]
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("", 348)]
    #[case("catpp", 8)]
    #[case("Catpp", 4)]
    #[case("catpp mocha", 2)]
    #[case("mocha catpp", 2)]
    #[case("catpp type:dark", 6)]
    #[case("catpp type:light", 2)]
    #[case("type:dark", 298)]
    #[case("type:light", 50)]
    #[case("ctpp", 0)]
    #[case("\"Solarized Dark\"", 4)]
    #[case("brightness:very-dark", 187)]
    #[case("brightness:dark", 106)]
    #[case("brightness:medium", 5)]
    #[case("brightness:light", 15)]
    #[case("brightness:very-light", 35)]
    #[case("bg:gray", 188)]
    #[case("bg:red-pink", 5)]
    #[case("bg:orange-brown", 9)]
    #[case("bg:yellow", 9)]
    #[case("bg:green", 7)]
    #[case("bg:cyan", 13)]
    #[case("bg:blue", 82)]
    #[case("bg:purple", 35)]
    #[case("contrast:low", 72)]
    #[case("contrast:medium", 147)]
    #[case("contrast:high", 129)]
    #[case("catpp brightness:very-dark bg:blue contrast:medium", 2)]
    #[case("catpp brightness:very-light bg:gray contrast:medium", 2)]
    #[case("brightness:very-dark bg:blue contrast:high", 15)]
    #[case("type:light brightness:very-dark", 0)]
    #[case("catpp bg:purple", 0)]
    #[case(
        "\"3024 Night\" type:dark brightness:very-dark bg:orange-brown contrast:medium",
        1
    )]
    fn searches_the_bundled_catalog(#[case] query: &str, #[case] count: usize) {
        let filter = Filter::parse(query).unwrap();
        assert_eq!(
            crate::theme::THEMES
                .iter()
                .filter(|t| filter.matches(t, None))
                .count(),
            count
        );
        assert_eq!(Filter::parse(&filter.query()).unwrap(), filter);
        assert_eq!(
            Filter::from_fields(
                &filter.name_query(),
                filter.appearance,
                filter.favorite,
                filter.ignored,
                filter.brightness,
                filter.background_color,
                filter.text_contrast
            )
            .unwrap(),
            filter
        );
    }

    #[rstest]
    #[case("type:")]
    #[case("type:Dark")]
    #[case("type:dark type:dark")]
    #[case("type:dark type:light")]
    #[case("favorite:yes")]
    #[case("favorite:true favorite:false")]
    #[case("ignore:")]
    #[case("ignore:yes")]
    #[case("ignore:all ignore:all")]
    #[case("brightness:")]
    #[case("brightness:very_dark")]
    #[case("brightness:all")]
    #[case("bg:")]
    #[case("bg:Blue")]
    #[case("bg:unknown")]
    #[case("contrast:")]
    #[case("contrast:7")]
    #[case("contrast:high contrast:low")]
    #[case("bg:blue bg:purple")]
    #[case("brightness:dark brightness:dark")]
    #[case("type:dark bg:blue contrast:unknown")]
    #[case("\"catpp")]
    #[case("\"catpp\"mocha")]
    #[case("cat\"pp")]
    #[case("\"bad\\q\"")]
    fn rejects_incomplete_or_unsupported_queries(#[case] query: &str) {
        assert!(Filter::parse(query).is_err());
    }

    #[rstest]
    #[case("catpp ppu", "Catppuccin", Some(vec![0..6]))]
    #[case("ana", "banana", Some(vec![1..6]))]
    #[case("é", "Éclair", Some(vec![0..2]))]
    #[case("i", "İris", Some(vec![0..2, 3..4]))]
    #[case("Mocha", "catppuccin-mocha", None)]
    #[case("", "any", Some(vec![]))]
    fn returns_original_ranges_for_highlighting(
        #[case] query: &str,
        #[case] name: &str,
        #[case] expected: Option<Vec<Range<usize>>>,
    ) {
        assert_eq!(Filter::parse(query).unwrap().name_matches(name), expected);
    }

    #[test]
    fn quoted_literals_round_trip_without_becoming_filters() {
        let query = r#""type:dark" "bg:blue" "brightness:very-dark" "contrast:high" "a\"b" "c\\d" "two  spaces" """#;
        let filter = Filter::parse(query).unwrap();
        assert_eq!(filter.appearance, Appearance::All);
        assert_eq!(filter.brightness, None);
        assert_eq!(filter.background_color, None);
        assert_eq!(filter.text_contrast, None);
        assert_eq!(Filter::parse(&filter.query()).unwrap(), filter);
        assert_eq!(
            Filter::from_fields(
                &filter.name_query(),
                Appearance::All,
                Favorite::All,
                Ignored::Hide,
                None,
                None,
                None
            )
            .unwrap(),
            filter
        );
        assert!(
            Filter::from_fields(
                "type:dark",
                Appearance::All,
                Favorite::All,
                Ignored::Hide,
                None,
                None,
                None
            )
            .is_err()
        );
    }
}
