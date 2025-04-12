use chrono::{DateTime, Local, Timelike};
use serde::Deserialize;
use std::{
    fmt::{self, Display, Formatter},
    time::{Duration, Instant},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize)]
pub enum TimeFormat {
    HourMinSec,
    HourMinSecMSec,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum OpMode {
    Clock,
    CountDownTimer(Duration),
    CountUpTimer,
}

#[derive(Debug)]
pub struct Time {
    format: TimeFormat,
    mode: OpMode,
    start_time: Instant,
    stop_time: Option<Instant>,
    started: bool,
}

impl Time {
    const MINUTE_SECS: u64 = 60;
    const HOUR_SECS: u64 = Self::MINUTE_SECS * 60;

    pub fn new(format: TimeFormat) -> Self {
        Self {
            format,
            mode: OpMode::Clock,
            start_time: Instant::now(),
            stop_time: None,
            started: false,
        }
    }

    pub fn set_mode(&mut self, mode: OpMode) {
        if self.mode != mode {
            self.mode = mode;

            match self.mode {
                OpMode::CountDownTimer(_) | OpMode::CountUpTimer => {
                    self.start_time = Instant::now();
                }
                _ => (),
            }
        }
    }

    pub fn toggle_timer(&mut self) {
        match self.mode {
            OpMode::CountDownTimer(duration) => {
                self.started = !self.started;

                if self.started {
                    self.start_time = match self.stop_time {
                        Some(time) => {
                            if time - self.start_time > duration {
                                // Has been previously stopped and stopped has target duration
                                self.stop_time = None;

                                time
                            } else {
                                self.start_time
                            }
                        }
                        None => Instant::now(),
                    };
                } else {
                    self.stop_time = Some(Instant::now())
                }
            }
            OpMode::CountUpTimer => {
                self.started = !self.started;

                if self.started {
                    self.start_time = match self.stop_time {
                        Some(time) => {
                            self.stop_time = None;

                            time
                        }
                        None => Instant::now(),
                    };
                } else {
                    self.stop_time = Some(Instant::now())
                }
            }
            _ => (),
        }
    }

    pub fn mode(&self) -> OpMode {
        self.mode.clone()
    }

    pub fn current_time(&self) -> String {
        match self.mode {
            OpMode::CountDownTimer(duration) => {
                let delta = if !self.started {
                    if let Some(time) = self.stop_time {
                        time - self.start_time
                    } else {
                        Duration::new(0, 0)
                    }
                } else {
                    Instant::now() - self.start_time
                };
                let delta = if delta > duration {
                    Duration::new(0, 0)
                } else {
                    duration - delta
                };

                self.duration_to_str(&delta)
            }
            OpMode::CountUpTimer => {
                let delta = Instant::now() - self.start_time;

                self.duration_to_str(&delta)
            }
            OpMode::Clock => self.time_to_str(&Local::now()),
        }
    }

    fn time_to_str(&self, time: &DateTime<Local>) -> String {
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

    fn duration_to_str(&self, delta: &Duration) -> String {
        let hour = (delta.as_secs() / Self::HOUR_SECS) as u32;
        let minute = (delta.as_secs() / Self::MINUTE_SECS) as u32;
        let second = (delta.as_secs() % Self::MINUTE_SECS) as u32;
        match self.format {
            TimeFormat::HourMinSec => {
                let hour = Self::format_to_digit(2, hour);
                let minute = Self::format_to_digit(2, minute);
                let second = Self::format_to_digit(2, second);

                format!("{}:{}:{}", hour, minute, second)
            }
            TimeFormat::HourMinSecMSec => {
                let hour = Self::format_to_digit(2, hour);
                let minute = Self::format_to_digit(2, minute);
                let second = Self::format_to_digit(2, second);
                let millisecond = Self::format_to_digit(3, (delta.as_millis() % 1000) as u32);

                format!("{}:{}:{}.{}", hour, minute, second, millisecond)
            }
        }
    }

    fn format_to_digit(digit: u32, value: u32) -> String {
        if digit <= 1 {
            return value.to_string();
        }

        let mut prefix = String::new();

        for i in 1..digit.into() {
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
