// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

use std::{
    env,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Default, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct NeedleArgs {
    /// Display help message
    #[arg(long, short)]
    pub help: bool,

    /// Display version information
    #[arg(long, short)]
    pub version: bool,

    /// Generate default config
    #[arg(long, default_value_t = String::new())]
    pub gen_config: String,

    /// Print default config to stdout
    #[arg(long, short)]
    pub print: bool,

    /// Path for config file
    #[arg(long, short, default_value_t = String::new())]
    pub config: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AppState {
    Run,
    Help,
    Version,
    GenerateConfig(String),
    ConfigFilePath(String),
}

impl AppState {
    #[cfg(windows)]
    const NEWLINE: &'static str = "\r\n";
    #[cfg(not(windows))]
    const NEWLINE: &'static str = "\n";
    const MAX_ARGUMENTS: usize = 5;
    pub fn new(args: &NeedleArgs) -> Vec<Self> {
        let mut app_states = Vec::with_capacity(Self::MAX_ARGUMENTS);

        if args.help {
            app_states.push(Self::Help);
        }

        if args.version {
            app_states.push(Self::Version);
        }

        if args.print ^ !args.gen_config.is_empty() {
            app_states.push(Self::GenerateConfig(if args.print {
                "stdout".to_string()
            } else {
                args.gen_config.clone()
            }));
        }

        if !args.config.is_empty() {
            app_states.push(Self::ConfigFilePath(args.config.clone()));
        }

        app_states.push(Self::Run);

        app_states
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Run
    }
}

impl Display for AppState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::Run | Self::ConfigFilePath(_) | Self::GenerateConfig(_) => String::new(),
            Self::Version => {
                let app_name = env!("CARGO_PKG_NAME");
                let app_version = env!("CARGO_PKG_VERSION");
                let core_info = needle_core::version_info();

                format!("{app_name} {app_version} ({core_info})")
            }
            Self::Help => {
                let lines = [
                    "Usage: needle [OPTIONS]",
                    "",
                    "Options:",
                    "   -h, --help                  Display this message",
                    "   -c, --config [FILENAME]     Specify a path to a custom config file",
                    "                               If path of config file is not specified, it will default to default path",
                    "   -p, --print                 Output config default values to stdout",
                    "       --gen-config [FILENAME] Generates config file",
                    "                               If path is specified, config file is generated to the specified path",
                    "                                   Default path:",
                    "                                   - Linux: $HOME/.config/needle/config.toml",
                    "                                   - Windows: %AppData%\\Roaming\\bonohub13\\needle\\config\\config.toml",
                    "   -v, --version               Print version info and exit",
                ];

                lines
                    .iter()
                    .map(|s| format!("{}{}", s, Self::NEWLINE))
                    .collect()
            }
        };

        write!(f, "{msg}")
    }
}
