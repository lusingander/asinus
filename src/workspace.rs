#[cfg(test)]
use std::path::PathBuf;
use std::{collections::HashSet, io};

use ratatui::layout::Direction;
use serde::{Deserialize, Serialize};

use crate::{
    preview::{ContentSize, Preset},
    search::Filter,
    storage::DataFile,
};

pub const WORKSPACE_PATH_ENV: &str = "ASINUS_WORKSPACE_PATH";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompareLayout {
    #[default]
    Horizontal,
    Vertical,
}

impl From<Direction> for CompareLayout {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Horizontal => Self::Horizontal,
            Direction::Vertical => Self::Vertical,
        }
    }
}

impl From<CompareLayout> for Direction {
    fn from(value: CompareLayout) -> Self {
        match value {
            CompareLayout::Horizontal => Self::Horizontal,
            CompareLayout::Vertical => Self::Vertical,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub query: Option<String>,
    pub preset: Preset,
    #[serde(default)]
    pub preview_content: ContentSize,
    pub layout: CompareLayout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedFilter {
    pub name: String,
    pub query: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    version: u32,
    pub saved_filters: Vec<SavedFilter>,
    pub session: Session,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            version: 1,
            saved_filters: Vec::new(),
            session: Session::default(),
        }
    }
}

impl Workspace {
    fn parse(text: &str) -> Option<Self> {
        let mut workspace: Self = toml::from_str(text).ok()?;
        if workspace.version != 1 {
            return None;
        }
        let mut names = HashSet::new();
        for saved in &mut workspace.saved_filters {
            if normalized_name(&saved.name).ok()? != saved.name || !names.insert(saved.name.clone())
            {
                return None;
            }
            saved.query = Filter::parse(&saved.query).ok()?.query();
        }
        if let Some(query) = &mut workspace.session.query {
            *query = Filter::parse(query).ok()?.query();
        }
        Some(workspace)
    }

    pub fn name(&self, input: &str, except: Option<&str>) -> Result<String, String> {
        let name = normalized_name(input)?;
        if self
            .saved_filters
            .iter()
            .any(|saved| saved.name == name && Some(saved.name.as_str()) != except)
        {
            return Err("Name already exists".into());
        }
        Ok(name)
    }

    pub fn changed(&self, change: &WorkspaceChange) -> Result<Self, String> {
        let mut next = self.clone();
        match change {
            WorkspaceChange::New { name, query } => {
                let name = self.name(name, None)?;
                next.saved_filters.push(SavedFilter {
                    name,
                    query: Filter::parse(query)?.query(),
                });
            }
            WorkspaceChange::Rename { from, to } => {
                let name = self.name(to, Some(from))?;
                let saved = next
                    .saved_filters
                    .iter_mut()
                    .find(|s| &s.name == from)
                    .ok_or("Filter no longer exists")?;
                saved.name = name;
            }
            WorkspaceChange::Delete { name } => {
                let index = next
                    .saved_filters
                    .iter()
                    .position(|s| &s.name == name)
                    .ok_or("Filter no longer exists")?;
                next.saved_filters.remove(index);
            }
        }
        Ok(next)
    }
}

pub fn normalized_name(input: &str) -> Result<String, String> {
    let name = input.trim();
    if name.is_empty() {
        return Err("Enter a name".into());
    }
    if name.chars().count() > 40 {
        return Err("Use 1–40 characters".into());
    }
    if name.chars().any(char::is_control) {
        return Err("Control characters are not allowed".into());
    }
    Ok(name.into())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceChange {
    New { name: String, query: String },
    Rename { from: String, to: String },
    Delete { name: String },
}

impl WorkspaceChange {
    pub fn message(&self) -> String {
        match self {
            Self::New { name, .. } => format!("{name}: Filter saved"),
            Self::Rename { to, .. } => format!("{to}: Filter renamed"),
            Self::Delete { name } => format!("{name}: Filter deleted"),
        }
    }
}

pub enum WorkspaceAction {
    Session,
    Filter(WorkspaceChange),
}

pub struct WorkspaceStore {
    file: DataFile,
}

impl WorkspaceStore {
    pub fn from_env() -> io::Result<Self> {
        Ok(Self {
            file: DataFile::from_env(WORKSPACE_PATH_ENV, "workspace")?,
        })
    }

    #[cfg(test)]
    pub fn open(path: PathBuf) -> Self {
        Self {
            file: DataFile::open(Some(path), "workspace"),
        }
    }

    pub fn load(&mut self) -> (Workspace, Option<String>) {
        self.file.load(Workspace::parse)
    }

    pub fn save(&self, workspace: &Workspace) -> Result<(), String> {
        self.file
            .save(&toml::to_string_pretty(workspace).expect("serializable workspace"))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn saved_filters_round_trip_and_rename_delete_preserve_the_session() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.toml");
        let mut store = WorkspaceStore::open(path.clone());
        let (mut value, error) = store.load();
        assert!(error.is_none());
        assert!(!path.exists());
        for (name, query) in [
            ("Dark", "type:dark"),
            (
                "引用句",
                r#""Solarized Dark" favorite:true ignore:all brightness:very-dark bg:blue contrast:medium"#,
            ),
        ] {
            value = value
                .changed(&WorkspaceChange::New {
                    name: name.into(),
                    query: query.into(),
                })
                .unwrap();
        }
        value.session.query =
            Some("type:light ignore:false brightness:very-light bg:gray contrast:medium".into());
        value.session.preview_content = ContentSize::Large;
        store.save(&value).unwrap();
        value = value
            .changed(&WorkspaceChange::Rename {
                from: "Dark".into(),
                to: "夜".into(),
            })
            .unwrap();
        assert_eq!(value.saved_filters[0].name, "夜");
        assert_eq!(
            value.session.query.as_deref(),
            Some("type:light ignore:false brightness:very-light bg:gray contrast:medium")
        );
        value = value
            .changed(&WorkspaceChange::Delete { name: "夜".into() })
            .unwrap();
        assert_eq!(value.saved_filters.len(), 1);
        assert_eq!(
            value.session.query.as_deref(),
            Some("type:light ignore:false brightness:very-light bg:gray contrast:medium")
        );
        store.save(&value).unwrap();
        drop(store);
        assert_eq!(WorkspaceStore::open(path).load().0, value);
    }

    #[rstest]
    #[case("version", toml::Value::Integer(2))]
    #[case("unknown", toml::Value::Boolean(true))]
    #[case("session", toml::toml! { query = "type:" preset = "palette" layout = "horizontal" }.into())]
    #[case("session", toml::toml! { query = "bg:blue contrast:unknown" preset = "palette" layout = "horizontal" }.into())]
    #[case("session", toml::toml! { preset = "unknown" layout = "horizontal" }.into())]
    #[case("session", toml::toml! { preset = "palette" layout = "unknown" }.into())]
    #[case("session", toml::toml! { preset = "palette" layout = "horizontal" preview_content = "unknown" }.into())]
    #[case("session", toml::toml! { preset = "palette" layout = "horizontal" unknown = true }.into())]
    #[case("saved_filters", toml::Value::try_from(vec![SavedFilter { name: "trim ".into(), query: "".into() }]).unwrap())]
    #[case("saved_filters", toml::Value::try_from(vec![SavedFilter { name: "bad\u{7f}name".into(), query: "".into() }]).unwrap())]
    #[case("saved_filters", toml::Value::try_from(vec![SavedFilter { name: "Bad".into(), query: "type:".into() }]).unwrap())]
    #[case("saved_filters", toml::Value::try_from(vec![SavedFilter { name: "Bad color".into(), query: "type:dark brightness:very-dark bg:unknown".into() }]).unwrap())]
    #[case("saved_filters", toml::Value::try_from(vec![SavedFilter { name: "Same".into(), query: "".into() }; 2]).unwrap())]
    fn invalid_workspace_is_rejected(#[case] field: &str, #[case] invalid: toml::Value) {
        let mut value = toml::Value::try_from(Workspace::default()).unwrap();
        value.as_table_mut().unwrap().insert(field.into(), invalid);
        let text = toml::to_string(&value).unwrap();
        assert!(Workspace::parse(&text).is_none());
    }

    #[test]
    fn names_trim_whitespace_count_unicode_characters_and_reject_duplicates() {
        let name = "夜".repeat(40);
        assert_eq!(normalized_name(&format!("  {name}  ")).unwrap(), name);
        assert!(normalized_name(&(name.clone() + "界")).is_err());
        assert!(normalized_name(" \t ").is_err());
        let workspace = Workspace::default()
            .changed(&WorkspaceChange::New {
                name: "  Dark  ".into(),
                query: "type:dark".into(),
            })
            .unwrap();
        assert_eq!(workspace.saved_filters[0].name, "Dark");
        assert!(workspace.name(" Dark ", None).is_err());
        assert_eq!(workspace.name(" Dark ", Some("Dark")).unwrap(), "Dark");
        assert_eq!(workspace.name("dark", None).unwrap(), "dark");
    }

    #[test]
    fn workspace_without_content_size_restores_auto_and_preserves_other_settings() {
        let workspace = Workspace::parse("version = 1\nsaved_filters = []\n[session]\npreset = 'git'\nlayout = 'vertical'\nquery = 'type:light'\n").unwrap();
        assert_eq!(workspace.session.preview_content, ContentSize::Auto);
        assert_eq!(workspace.session.preset, Preset::Git);
        assert_eq!(workspace.session.layout, CompareLayout::Vertical);
        assert_eq!(
            workspace.session.query.as_deref(),
            Some("type:light ignore:false")
        );
    }
}
