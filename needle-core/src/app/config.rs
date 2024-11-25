use crate::{error::NeedleError, TimeFormat};
use anyhow::Result;
use directories::ProjectDirs;
use serde::Deserialize;
use std::{
    fmt::{self, Display, Formatter},
    fs::{self, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct NeedleConfig {
    pub text: Text,
    pub background_color: [f64; 4],
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Text {
    pub format: TimeFormat,
    pub scale: f32,
    pub color: [u8; 4],
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
            let config_file = config_file.to_string_lossy();

            return Err(NeedleError::ConfigNonExistant(config_file.into()).into());
        }

        let read = OpenOptions::new().read(true).open(config_file)?;
        let mut buf_reader = BufReader::new(read);
        let mut read_buffer = String::new();

        buf_reader.read_to_string(&mut read_buffer)?;

        let config = toml::from_str(&read_buffer)?;

        Ok(config)
    }

    fn config_file(create_dir: bool) -> Result<PathBuf> {
        match ProjectDirs::from("com", "bonohub13", "needle") {
            Some(app_dir) => {
                if (!app_dir.config_dir().exists()) && create_dir {
                    fs::create_dir(app_dir.config_dir())?;
                }

                Ok(app_dir.config_dir().join(Self::CONFIG_FILE))
            }
            None => Err(NeedleError::InvalidPath.into()),
        }
    }

    fn write(file: &Path) -> Result<()> {
        if file.exists() {
            return Err(NeedleError::ConfigExists.into());
        }

        let file = OpenOptions::new().write(true).create_new(true).open(file)?;
        let mut buf_writer = BufWriter::new(file);
        let config = Self::default();

        Ok(writeln!(buf_writer, "{}", config)?)
    }
}

impl Default for NeedleConfig {
    fn default() -> Self {
        Self {
            text: Text {
                format: TimeFormat::HourMinSec,
                scale: 1.0,
                color: [255, 255, 255, 255],
            },
            background_color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Display for NeedleConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        /* Config:
         *
         * ---
         *
         * background = [r, g, b, alpha]
         *
         * [text]
         * # Time format
         * #    HourMinSec : HH:MM:SS (default)
         * #    HourMinSecMSec : HH:MM:SS.MSec
         * format = time format
         * scale = text scale size
         * color = [r, g, b, alpha]
         */

        writeln!(f, "# Background color : [r, g, b, alpha]")?;
        writeln!(
            f,
            "background_color = [{}, {}, {}, {}]",
            self.background_color[0],
            self.background_color[1],
            self.background_color[2],
            self.background_color[3]
        )?;
        writeln!(f, "# Text Settings")?;
        writeln!(f, "{}[text]", Self::NEWLINE)?;
        writeln!(f, "#  Time format")?;
        writeln!(f, "#      HourMinSec : HH:MM:SS (default)")?;
        writeln!(f, "#      HourMinSecMSec : HH:MM:SS.MSec")?;
        writeln!(f, "format = \"{}\"", self.text.format)?;
        writeln!(f, "#  Text scale")?;
        writeln!(f, "scale = {}", self.text.scale)?;
        writeln!(f, "# Text color : [r, g, b, alpha]")?;
        writeln!(
            f,
            "color = [{}, {}, {}, {}]",
            self.text.color[0], self.text.color[1], self.text.color[2], self.text.color[3]
        )
    }
}
