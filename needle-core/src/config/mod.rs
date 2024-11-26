mod fps;
mod position;
mod text;
mod time;

pub use fps::*;
pub use position::*;
pub use text::*;
pub use time::*;

use crate::{error::NeedleError, TimeFormat};
use anyhow::{bail, Result};
use directories::ProjectDirs;
use serde::Deserialize;
use std::{
    ffi::OsStr,
    fmt::{self, Display, Formatter},
    fs::{self, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct NeedleConfig {
    pub background_color: [f64; 4],
    pub time: TimeConfig,
    pub fps: FpsConfig,
}

impl NeedleConfig {
    #[cfg(windows)]
    const NEWLINE: &str = "\r\n";
    #[cfg(not(windows))]
    const NEWLINE: &str = "\n";
    const CONFIG_FILE: &str = "config.toml";

    pub fn config(path: Option<&str>) -> Result<()> {
        let default_config_file = Self::config_file(true)?;
        let config_file = if let Some(path) = path {
            if path.is_empty() {
                &default_config_file
            } else {
                Path::new(path)
            }
        } else {
            &default_config_file
        };

        Self::write(&config_file)
    }

    pub fn from(path: Option<&str>) -> Result<Self> {
        let default_config_file = Self::config_file(false)?;
        let config_file = if let Some(path) = path {
            if path.is_empty() {
                &default_config_file
            } else {
                Path::new(path)
            }
        } else {
            &default_config_file
        };

        if !config_file.exists() {
            if config_file == &default_config_file {
                Self::config(None)?;
            } else {
                let config_file = config_file.to_string_lossy();

                bail!(NeedleError::ConfigNonExistant(config_file.into()))
            }
        }

        let read = OpenOptions::new().read(true).open(config_file)?;
        let mut buf_reader = BufReader::new(read);
        let mut read_buffer = String::new();

        buf_reader.read_to_string(&mut read_buffer)?;

        let config: Self = toml::from_str(&read_buffer)?;

        if config.fps.enable && !config.fps.is_valid_position() {
            bail!(NeedleError::InvalidFpsTextPosition(
                config.fps.config.position
            ))
        } else if config.fps.enable && (config.fps.config.position == config.time.config.position) {
            bail!(NeedleError::TextPositionOverlapping)
        } else {
            Ok(config)
        }
    }

    fn config_file(create_dir: bool) -> Result<PathBuf> {
        match ProjectDirs::from("com", "bonohub13", "needle") {
            Some(app_dir) => {
                if (!app_dir.config_dir().exists()) && create_dir {
                    fs::create_dir(app_dir.config_dir())?;
                }

                Ok(app_dir.config_dir().join(Self::CONFIG_FILE))
            }
            None => bail!(NeedleError::InvalidPath),
        }
    }

    fn write(file: &Path) -> Result<()> {
        let default_config_path = Self::config_file(false)?;
        if file.exists() && file == default_config_path.as_path() {
            bail!(NeedleError::ConfigExists)
        }

        let config = Self::default();

        if file.as_os_str() == OsStr::new("stdout") {
            println!("{}", config);

            Ok(())
        } else {
            let file = OpenOptions::new().write(true).create(true).open(file)?;
            let mut buf_writer = BufWriter::new(file);

            Ok(writeln!(buf_writer, "{}", config)?)
        }
    }
}

impl Default for NeedleConfig {
    fn default() -> Self {
        Self {
            background_color: [0.0, 0.0, 0.0, 1.0],
            time: TimeConfig {
                format: TimeFormat::HourMinSec,
                config: Text {
                    scale: 1.0,
                    color: [255, 255, 255, 255],
                    position: Position::Center,
                },
            },
            fps: FpsConfig {
                enable: false,
                config: Text {
                    scale: 0.25,
                    color: [255, 0, 0, 255],
                    position: Position::TopRight,
                },
            },
        }
    }
}

impl Display for NeedleConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "# Background color : [r, g, b, alpha]")?;
        writeln!(f, "#  Range : (0.0 - 1.0)")?;
        writeln!(
            f,
            "background_color = [{}, {}, {}, {}]",
            self.background_color[0],
            self.background_color[1],
            self.background_color[2],
            self.background_color[3]
        )?;
        writeln!(f, "{}[time]", Self::NEWLINE)?;
        writeln!(f, "{}", self.time)?;
        writeln!(f, "{}[fps]", Self::NEWLINE)?;
        write!(f, "{}", self.fps)
    }
}
