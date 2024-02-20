use serde::Deserialize;
use std::{env, fs, path::PathBuf};

// TODO: Config the cli tools to use
#[derive(Deserialize)]
pub struct Config {
    pub editor: Option<String>,
    pub notes_dir: PathBuf,
}

impl Config {
    pub fn new(path: Option<PathBuf>) -> Self {
        toml::from_str(
            match &fs::read_to_string(path.unwrap_or_else(Self::get_path)) {
                Ok(content) => content,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    eprintln!("Config file not found");
                    eprintln!("Using the default configuration");
                    return Self::default();
                }
                Err(_) => {
                    eprintln!("Failed to read config file");
                    eprintln!("Using the default configuration");
                    return Self::default();
                }
            },
        )
        .unwrap_or_default()
    }

    fn get_path() -> PathBuf {
        env::var_os("NORMD_CONFIG_HOME").map_or_else(
            || dirs::config_dir().unwrap().join("normd/config.toml"),
            |path| PathBuf::from(path).join("config.toml"),
        )
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: None,
            notes_dir: dirs::home_dir().unwrap().join("notes/"),
        }
    }
}
