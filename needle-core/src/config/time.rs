use super::Text;
use crate::TimeFormat;
use serde::Deserialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct TimeConfig {
    pub format: TimeFormat,
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
