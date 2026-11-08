mod style;

use std::{
    env,
    error::Error,
    ffi::OsString,
    fmt, fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use umbra::optional;

use crate::{config::style::StyleConfig, keybindings::KeyBindings, ui::theme::UiTheme};

const CONFIG_FILE_ENV: &str = "ASINUS_CONFIG_FILE";

#[derive(Default)]
pub struct Config {
    pub ui_theme: UiTheme,
    pub keybindings: KeyBindings,
}

#[optional(derives = [Debug, Deserialize], attrs = [serde(deny_unknown_fields)])]
#[derive(Default)]
struct FileConfig {
    #[nested]
    ui: UiConfig,
}

#[optional(derives = [Debug, Deserialize], attrs = [serde(deny_unknown_fields)])]
#[derive(Default)]
struct UiConfig {
    style: StyleConfig,
}

enum ConfigFile {
    Explicit(PathBuf),
    Default(PathBuf),
}

#[derive(Debug)]
pub enum ConfigError {
    EmptyExplicitPath,
    ExplicitFileNotFound(PathBuf),
    Read {
        path: PathBuf,
        source: io::Error,
    },
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyExplicitPath => write!(formatter, "${CONFIG_FILE_ENV} must not be empty"),
            Self::ExplicitFileNotFound(path) => write!(
                formatter,
                "config file specified by ${CONFIG_FILE_ENV} was not found: {}",
                path.display()
            ),
            Self::Read { path, source } => {
                write!(
                    formatter,
                    "failed to read config file {}: {source}",
                    path.display()
                )
            }
            Self::Parse { path, source } => {
                write!(
                    formatter,
                    "failed to parse config file {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::EmptyExplicitPath | Self::ExplicitFileNotFound(_) => None,
        }
    }
}

pub fn load() -> Result<Config, ConfigError> {
    let config_file = resolve_config_file(
        env::var_os(CONFIG_FILE_ENV),
        env::var_os("XDG_CONFIG_HOME"),
        env::var_os("HOME"),
    )?;
    let Some(config_file) = config_file else {
        return Ok(Config::default());
    };
    load_file(config_file)
}

fn resolve_config_file(
    explicit: Option<OsString>,
    xdg: Option<OsString>,
    home: Option<OsString>,
) -> Result<Option<ConfigFile>, ConfigError> {
    if let Some(path) = explicit {
        if path.is_empty() {
            return Err(ConfigError::EmptyExplicitPath);
        }
        return Ok(Some(ConfigFile::Explicit(PathBuf::from(path))));
    }

    let config_home = xdg
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            home.filter(|path| !path.is_empty())
                .map(|path| PathBuf::from(path).join(".config"))
        });
    Ok(config_home.map(|path| ConfigFile::Default(path.join("asinus/config.toml"))))
}

fn load_file(file: ConfigFile) -> Result<Config, ConfigError> {
    let (path, explicit) = match &file {
        ConfigFile::Explicit(path) => (path.as_path(), true),
        ConfigFile::Default(path) => (path.as_path(), false),
    };
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(source) if source.kind() == ErrorKind::NotFound && !explicit => {
            return Ok(Config::default());
        }
        Err(source) if source.kind() == ErrorKind::NotFound => {
            return Err(ConfigError::ExplicitFileNotFound(path.to_owned()));
        }
        Err(source) => {
            return Err(ConfigError::Read {
                path: path.to_owned(),
                source,
            });
        }
    };
    parse(&content, path)
}

fn parse(content: &str, path: &Path) -> Result<Config, ConfigError> {
    let optional =
        toml::from_str::<OptionalFileConfig>(content).map_err(|source| ConfigError::Parse {
            path: path.to_owned(),
            source,
        })?;
    let config = FileConfig::from(optional);
    Ok(Config {
        ui_theme: config.ui.style.resolve(),
        keybindings: KeyBindings::default(),
    })
}
