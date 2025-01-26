use super::{Position, Text};
use serde::Deserialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Deserialize)]
pub struct FpsConfig {
    pub enable: bool,
    pub frame_limit: u8,
    pub config: Text,
}

impl FpsConfig {
    pub fn is_valid_position(&self) -> bool {
        match self.config.position {
            Position::TopLeft
            | Position::TopRight
            | Position::BottomLeft
            | Position::BottomRight => true,
            _ => false,
        }
    }
}

impl Display for FpsConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let config = format!("{}", self.config);
        let config = config.lines().into_iter().collect::<Vec<_>>();

        writeln!(f, "# FPS visualization setting")?;
        writeln!(f, "#  true            : Enable FPS visualization")?;
        writeln!(f, "#  false (default) : Disable FPS visualization")?;
        writeln!(f, "enable = {}", if self.enable { "true" } else { "false" })?;
        writeln!(f, "# FPS limit")?;
        writeln!(f, "frame_limit = {}", self.frame_limit)?;
        for (i, line) in config.iter().enumerate() {
            if line.starts_with("#") {
                if i == (config.len() - 1) {
                    return write!(f, "{}", line);
                } else {
                    writeln!(f, "{}", line)?;
                }
            } else {
                if i == (config.len() - 1) {
                    return write!(f, "config.{}", line);
                } else {
                    writeln!(f, "config.{}", line)?;
                }
            }
        }

        Ok(())
    }
}
