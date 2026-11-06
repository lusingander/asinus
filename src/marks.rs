#[cfg(test)]
use std::path::PathBuf;
use std::{collections::BTreeMap, io};

use serde::{Deserialize, Serialize};

use crate::storage::DataFile;

pub const STATE_PATH_ENV: &str = "ASINUS_STATE_PATH";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mark {
    Favorite,
    Ignored,
}

impl Mark {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Favorite => "+",
            Self::Ignored => "-",
        }
    }
}

pub type Marks = BTreeMap<String, Mark>;

#[derive(Debug, Clone)]
pub struct MarkChange {
    pub name: String,
    pub mark: Option<Mark>,
}

impl MarkChange {
    pub fn apply(&self, marks: &mut Marks) {
        if let Some(mark) = self.mark {
            marks.insert(self.name.clone(), mark);
        } else {
            marks.remove(&self.name);
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredMarks {
    version: u32,
    themes: Marks,
}

impl StoredMarks {
    fn parse(text: &str) -> Option<Marks> {
        let stored: Self = toml::from_str(text).ok()?;
        if stored.version != 1
            || stored
                .themes
                .keys()
                .any(|name| name.is_empty() || name.trim() != name)
        {
            return None;
        }
        Some(stored.themes)
    }
}

pub struct MarkStore {
    file: DataFile,
}

impl MarkStore {
    pub fn from_env() -> io::Result<Self> {
        Ok(Self {
            file: DataFile::from_env(STATE_PATH_ENV, "marks")?,
        })
    }

    #[cfg(test)]
    pub fn open(path: PathBuf) -> Self {
        Self {
            file: DataFile::open(Some(path), "marks"),
        }
    }

    pub fn load(&mut self) -> (Marks, Option<String>) {
        self.file.load(StoredMarks::parse)
    }

    pub fn save(&self, marks: &Marks) -> Result<(), String> {
        let text = toml::to_string_pretty(&StoredMarks {
            version: 1,
            themes: marks.clone(),
        })
        .expect("marks contain only serializable names and states");
        self.file.save(&text)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rstest::rstest;

    use super::*;

    #[test]
    fn locked_store_can_read_marks_but_cannot_overwrite_them() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("marks.toml");
        let mut writer = MarkStore::open(path.clone());
        assert!(writer.load().1.is_none());
        assert!(!path.exists());
        let marks = Marks::from([("Removed Theme".into(), Mark::Favorite)]);
        writer.save(&marks).unwrap();
        let mut reader = MarkStore::open(path.clone());
        let (loaded, error) = reader.load();
        assert_eq!(loaded, marks);
        assert!(error.unwrap().contains("another instance"));
        assert!(reader.save(&Marks::new()).is_err());
        drop(writer);
        let (loaded, error) = MarkStore::open(path).load();
        assert!(error.is_none());
        assert_eq!(loaded, marks);
    }

    #[rstest]
    #[case(
        r#"
invalid = [
"#
    )]
    #[case(
        r#"
version = 2
[themes]
"#
    )]
    #[case(
        r#"
version = 1
[themes]
Dracula = "unknown"
"#
    )]
    #[case(
        r#"
version = 1
[themes]
Dracula = "favorite"
Dracula = "ignored"
"#
    )]
    #[case(
        r#"
version = 1
version = 1
[themes]
"#
    )]
    #[case(
        r#"
version = 1
[themes]
"" = "favorite"
"#
    )]
    #[case(
        r#"
version = 1
[themes]
" Dracula" = "favorite"
"#
    )]
    fn invalid_marks_are_rejected(#[case] text: &str) {
        assert!(StoredMarks::parse(text).is_none());
    }

    #[test]
    fn failed_replacement_keeps_the_destination_and_removes_the_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("marks.toml");
        let mut store = MarkStore::open(path.clone());
        store.load();
        fs::create_dir(&path).unwrap();
        let sentinel = path.join("keep");
        fs::write(&sentinel, "unchanged").unwrap();
        assert!(store.save(&Marks::new()).is_err());
        assert_eq!(fs::read_to_string(sentinel).unwrap(), "unchanged");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }
}
