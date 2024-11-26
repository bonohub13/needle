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
    pub scale: f32,
    pub color: [u8; 4],
    pub format: TimeFormat,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Position {
    Center,
    Top,
    Bottom,
    Right,
    Left,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
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

                return Err(NeedleError::ConfigNonExistant(config_file.into()).into());
            }
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
        let default_config_path = Self::config_file(false)?;
        if file.exists() && file == default_config_path.as_path() {
            return Err(NeedleError::ConfigExists.into());
        }

        let file = OpenOptions::new().write(true).create(true).open(file)?;
        let mut buf_writer = BufWriter::new(file);
        let config = Self::default();

        Ok(writeln!(buf_writer, "{}", config)?)
    }
}

impl Default for NeedleConfig {
    fn default() -> Self {
        Self {
            text: Text {
                scale: 1.0,
                color: [255, 255, 255, 255],
                format: TimeFormat::HourMinSec,
                position: Position::Center,
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
         * # Background color : [r, g, b, alpha]
         * background = [r, g, b, alpha]
         *
         * # Text Settings
         * [text]
         * scale = text scale size
         * color = [r, g, b, alpha]
         * # Time format
         * #    HourMinSec : HH:MM:SS (default)
         * #    HourMinSecMSec : HH:MM:SS.MSec
         * format = time format
         */

        writeln!(f, "# Background color : [r, g, b, alpha]")?;
        writeln!(f, "#  Range : (0.0-1.0)")?;
        writeln!(
            f,
            "background_color = [{}, {}, {}, {}]",
            self.background_color[0],
            self.background_color[1],
            self.background_color[2],
            self.background_color[3]
        )?;
        writeln!(f, "{}# Text Settings", Self::NEWLINE)?;
        writeln!(f, "[text]")?;
        writeln!(f, "#  Text scale")?;
        writeln!(f, "scale = {}", self.text.scale)?;
        writeln!(f, "#  Text color : [r, g, b, alpha]")?;
        writeln!(f, "#      Range : (0-255)")?;
        writeln!(
            f,
            "color = [{}, {}, {}, {}]",
            self.text.color[0], self.text.color[1], self.text.color[2], self.text.color[3]
        )?;
        writeln!(f, "#  Time format")?;
        writeln!(f, "#      HourMinSec : HH:MM:SS (default)")?;
        writeln!(f, "#      HourMinSecMSec : HH:MM:SS.MSec")?;
        writeln!(f, "format = \"{}\"", self.text.format)?;
        writeln!(f, "#  Position")?;
        writeln!(f, "#      Center")?;
        writeln!(f, "#      Top")?;
        writeln!(f, "#      Bottom")?;
        writeln!(f, "#      Right")?;
        writeln!(f, "#      Left")?;
        writeln!(f, "#      TopRight")?;
        writeln!(f, "#      TopLeft")?;
        writeln!(f, "#      BottomRight")?;
        writeln!(f, "#      BottomLeft")?;
        writeln!(f, "position = {}", self.text.position)
    }
}

impl Text {
    const MARGIN: f32 = 5.0;
    pub fn position(
        &self,
        screen_size: &winit::dpi::PhysicalSize<u32>,
        text_size: &[f32; 2],
    ) -> (f32, f32) {
        match self.position {
            Position::Center => Self::center(screen_size, text_size),
            Position::Top => Self::top(screen_size, text_size),
            Position::Bottom => Self::bottom(screen_size, text_size),
            Position::Left => Self::left(screen_size, text_size),
            Position::Right => Self::right(screen_size, text_size),
            Position::TopLeft => {
                let top = Self::top(screen_size, text_size);
                let left = Self::left(screen_size, text_size);

                (left.0, top.1)
            }
            Position::TopRight => {
                let top = Self::top(screen_size, text_size);
                let right = Self::right(screen_size, text_size);

                (right.0, top.1)
            }
            Position::BottomLeft => {
                let bottom = Self::bottom(screen_size, text_size);
                let left = Self::left(screen_size, text_size);

                (left.0, bottom.1)
            }
            Position::BottomRight => {
                let bottom = Self::bottom(screen_size, text_size);
                let right = Self::right(screen_size, text_size);

                (right.0, bottom.1)
            }
        }
    }

    fn center(screen_size: &winit::dpi::PhysicalSize<u32>, text_size: &[f32; 2]) -> (f32, f32) {
        (
            (screen_size.width as f32 - text_size[0]) / 2.0,
            (screen_size.height as f32 - text_size[1]) / 2.0,
        )
    }

    fn top(screen_size: &winit::dpi::PhysicalSize<u32>, text_size: &[f32; 2]) -> (f32, f32) {
        (
            (screen_size.width as f32 - text_size[0]) / 2.0,
            Self::MARGIN * 2.0,
        )
    }

    fn bottom(screen_size: &winit::dpi::PhysicalSize<u32>, text_size: &[f32; 2]) -> (f32, f32) {
        (
            (screen_size.width as f32 - text_size[0]) / 2.0,
            screen_size.height as f32 - text_size[1] - (Self::MARGIN * 2.0),
        )
    }

    fn left(screen_size: &winit::dpi::PhysicalSize<u32>, text_size: &[f32; 2]) -> (f32, f32) {
        (
            Self::MARGIN,
            (screen_size.height as f32 - text_size[1]) / 2.0,
        )
    }

    fn right(screen_size: &winit::dpi::PhysicalSize<u32>, text_size: &[f32; 2]) -> (f32, f32) {
        (
            screen_size.width as f32 - text_size[0] - Self::MARGIN,
            (screen_size.height as f32 - text_size[1]) / 2.0,
        )
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let position = match self {
            Self::Center => "Center",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
            Self::Right => "Right",
            Self::Left => "Left",
            Self::TopRight => "TopRight",
            Self::TopLeft => "TopLeft",
            Self::BottomRight => "BottomRight",
            Self::BottomLeft => "BottomLeft",
        };

        write!(f, "\"{}\"", position)
    }
}
