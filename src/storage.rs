use std::{
    env,
    fs::{self, File, OpenOptions, TryLockError},
    io::{self, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

pub struct DataFile {
    path: Option<PathBuf>,
    lock: Option<File>,
    kind: &'static str,
}

impl DataFile {
    pub fn from_env(path_env: &'static str, kind: &'static str) -> io::Result<Self> {
        let path = if let Some(path) = env::var_os(path_env) {
            if path.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "empty state path",
                ));
            }
            Some(env::current_dir()?.join(path))
        } else {
            default_path(
                env::var_os("HOME").map(PathBuf::from),
                env::var_os("XDG_DATA_HOME").map(PathBuf::from),
                kind,
            )
        };
        Ok(Self::open(path, kind))
    }

    pub fn open(path: Option<PathBuf>, kind: &'static str) -> Self {
        Self {
            path,
            lock: None,
            kind,
        }
    }

    fn acquire_lock(&self, path: &Path) -> Result<File, String> {
        let parent = path.parent().ok_or("state path has no parent")?;
        fs::create_dir_all(parent).map_err(|error| error.kind().to_string())?;
        let mut lock_path = path.as_os_str().to_owned();
        lock_path.push(".lock");
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|error| error.kind().to_string())?;
        match lock.try_lock() {
            Ok(()) => Ok(lock),
            Err(TryLockError::WouldBlock) => Err("another instance holds the lock".into()),
            Err(TryLockError::Error(error)) => Err(error.kind().to_string()),
        }
    }

    pub fn load<T: Default>(
        &mut self,
        parse: impl FnOnce(&str) -> Option<T>,
    ) -> (T, Option<String>) {
        let Some(path) = self.path.clone() else {
            return (T::default(), Some("no state directory".into()));
        };
        let lock = match self.lock.take() {
            Some(lock) => Ok(lock),
            None => self.acquire_lock(&path),
        };
        let value = match fs::read_to_string(&path) {
            Ok(text) => match parse(&text) {
                Some(value) => value,
                None => return (T::default(), Some(format!("invalid {}.toml", self.kind))),
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => T::default(),
            Err(error) => return (T::default(), Some(error.kind().to_string())),
        };
        match lock {
            Ok(lock) => {
                self.lock = Some(lock);
                (value, None)
            }
            Err(error) => (value, Some(error)),
        }
    }

    pub fn save(&self, text: &str) -> Result<(), String> {
        let path = self.path.as_deref().ok_or("no state directory")?;
        if self.lock.is_none() {
            return Err("read-only".into());
        }
        self.write(path, text)
            .map_err(|error| error.kind().to_string())
    }

    fn write(&self, path: &Path, text: &str) -> io::Result<()> {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "state path has no parent")
        })?;
        let mut file = NamedTempFile::new_in(parent)?;
        file.write_all(text.as_bytes())?;
        file.flush()?;
        file.as_file().sync_all()?;
        file.persist(path).map_err(|e| e.error)?;
        Ok(())
    }
}

fn default_path(home: Option<PathBuf>, xdg: Option<PathBuf>, kind: &str) -> Option<PathBuf> {
    xdg.filter(|p| p.is_absolute())
        .or_else(|| {
            home.filter(|p| p.is_absolute())
                .map(|p| p.join(".local/share"))
        })
        .map(|p| p.join("asinus").join(format!("{kind}.toml")))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Some("/data"), Some("/home/user"), Some("/data/asinus/marks.toml"))]
    #[case(
        None,
        Some("/home/user"),
        Some("/home/user/.local/share/asinus/marks.toml")
    )]
    #[case(
        Some(""),
        Some("/home/user"),
        Some("/home/user/.local/share/asinus/marks.toml")
    )]
    #[case(
        Some("relative"),
        Some("/home/user"),
        Some("/home/user/.local/share/asinus/marks.toml")
    )]
    #[case(Some("/data"), None, Some("/data/asinus/marks.toml"))]
    #[case(None, None, None)]
    fn data_home_has_priority_and_falls_back_to_local_share(
        #[case] xdg: Option<&str>,
        #[case] home: Option<&str>,
        #[case] expected: Option<&str>,
    ) {
        assert_eq!(
            default_path(home.map(PathBuf::from), xdg.map(PathBuf::from), "marks"),
            expected.map(PathBuf::from)
        );
    }
}
