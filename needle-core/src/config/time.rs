use super::Text;
use crate::TimeFormat;
use serde::Deserialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Deserialize)]
pub struct TimeConfig {
    pub format: TimeFormat,
    pub font: Option<String>,
    pub config: Text,
}

impl Display for TimeConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let config = format!("{}", self.config);
        let config = config.lines().into_iter().collect::<Vec<_>>();

        writeln!(f, "# Time format")?;
        writeln!(f, "#  HourMinSec : HH:MM:SS (default)")?;
        writeln!(f, "#  HourMinSecMSec : HH:MM:SS.MSec")?;
        writeln!(f, "format = \"{}\"", self.format)?;
        writeln!(f, "# Fonts (Optional)")?;
        #[cfg(target_os = "windows")]
        writeln!(
            f,
            "#  Fonts installed under \"%APPDATA%\\bonohub13\\needle\\config\\fonts\" can be used."
        )?;
        #[cfg(target_os = "linux")]
        writeln!(
            f,
            "#  Fonts installed under \"${{HOME}}/.config/needle/fonts\" can be used."
        )?;
        writeln!(f, "#  Fonts must be either of \".otf\" or \".ttf\" file.")?;
        writeln!(f, "#  fonts = \"\" (default)")?;
        writeln!(f, "#  Example:")?;
        writeln!(f, "#      font = \"DejaVu Serif.ttf\"")?;
        writeln!(f, "font = \"\"")?;
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
