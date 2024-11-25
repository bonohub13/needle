use chrono::{DateTime, Local, Timelike};
use serde::Deserialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize)]
pub enum TimeFormat {
    HourMinSec,
    HourMinSecMSec,
}

#[derive(Debug)]
pub struct Time {
    format: TimeFormat,
}

impl Time {
    pub fn new(format: TimeFormat) -> Self {
        Self { format }
    }

    pub fn time_to_str(&self, time: &DateTime<Local>) -> String {
        match self.format {
            TimeFormat::HourMinSec => {
                let hour = Self::format_to_digit(2, time.hour());
                let minute = Self::format_to_digit(2, time.minute());
                let second = Self::format_to_digit(2, time.second());

                format!("{}:{}:{}", hour, minute, second)
            }
            TimeFormat::HourMinSecMSec => {
                let hour = Self::format_to_digit(2, time.hour());
                let minute = Self::format_to_digit(2, time.minute());
                let second = Self::format_to_digit(2, time.second());
                let millisecond = Self::format_to_digit(3, time.nanosecond() / 1_000_000);

                format!("{}:{}:{}.{}", hour, minute, second, millisecond)
            }
        }
    }

    fn format_to_digit(digit: u32, value: u32) -> String {
        if digit <= 1 {
            return value.to_string();
        }

        let mut prefix = String::new();

        for i in 1..digit {
            if value < 10u32.pow(i) {
                prefix = prefix + "0";
            }
        }

        format!("{}{}", prefix, value)
    }
}

impl Display for TimeFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let format = match self {
            TimeFormat::HourMinSec => "HourMinSec",
            TimeFormat::HourMinSecMSec => "HourMinSecMSec",
        };

        write!(f, "{}", format)
    }
}
