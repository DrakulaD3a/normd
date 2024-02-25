use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Deserialize)]
pub struct Config {
    pub editor: Option<String>,
    pub notes_dir: PathBuf,
    pub port: u16,
}

impl Config {
    pub fn new(path: Option<PathBuf>) -> anyhow::Result<Self> {
        let path = path.unwrap_or_else(Self::get_path);

        Ok(toml::from_str(match &fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                eprintln!("Config file not found");
                eprintln!("Using the default configuration");
                return Ok(Self::default());
            }
            Err(_) => {
                eprintln!("Failed to read config file");
                eprintln!("Using the default configuration");
                return Ok(Self::default());
            }
        })
        .unwrap_or_default())
    }

    fn get_path() -> PathBuf {
        PathBuf::from(env::var_os("NORMD_CONFIG_HOME").unwrap_or_else(|| {
            dirs::config_dir()
                .expect("Failed to get config directory")
                .join("normd")
                .into_os_string()
        }))
        .join("config.toml")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: None,
            notes_dir: dirs::home_dir()
                .unwrap_or_else(|| {
                    eprintln!("Failed to get home directory");
                    // TODO: Exit codes
                    std::process::exit(1);
                })
                .join("notes/"),
            port: 8080,
        }
    }
}
