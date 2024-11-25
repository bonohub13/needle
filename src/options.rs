use std::{
    env,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, PartialEq, Clone)]
pub enum AppOptions {
    Run,
    Help,
    Version,
    GenerateConfig(String),
    ConfigFilePath(String),
    Unknown(String),
}

impl AppOptions {
    #[cfg(windows)]
    const NEWLINE: &str = "\r\n";
    #[cfg(not(windows))]
    const NEWLINE: &str = "\n";
    pub fn new() -> Vec<Self> {
        let args = env::args().into_iter().collect::<Vec<_>>();
        let mut skip_counter = 0;
        let mut ret = vec![];

        for (i, arg) in args[1..].iter().enumerate() {
            if skip_counter > 0 {
                skip_counter -= 1;
                continue;
            }

            Self::parse_str(&arg).iter_mut().for_each(|opt| match opt {
                Self::ConfigFilePath(ref mut path) | Self::GenerateConfig(ref mut path) => {
                    if path == "" {
                        *path = if ((i + 2) < args.len()) && (args[i + 2] != "=") {
                            // OPTION FILENAME
                            skip_counter = 1;

                            args[i + 2].clone()
                        } else if (i + 3) < args.len() {
                            // OPTION = FILENAME
                            skip_counter = 2;

                            args[i + 3].clone()
                        } else {
                            // If empty path is detected, it fallbacks into default path for config
                            // file
                            path.clone()
                        }
                    }

                    ret.push(opt.clone());
                }
                _ => ret.push(opt.clone()),
            });
        }

        if ret.is_empty() {
            ret.push(Self::Run);
        }

        ret
    }

    fn parse_str(arg: &str) -> Vec<Self> {
        match arg {
            "--help" | "-h" => vec![Self::Help],
            "--version" | "-v" => vec![Self::Version],
            "--config" | "-c" => vec![Self::Run, Self::ConfigFilePath("".to_string())],
            "--gen-config" => vec![Self::GenerateConfig("".to_string())],
            "" | " " | "\t" => vec![Self::Run],
            _ => {
                let ret = if arg.starts_with("--config=") {
                    let src = arg.split("=").collect::<Vec<_>>();
                    let src = if src.len() == 1 {
                        ""
                    } else {
                        src.last().unwrap_or(&"")
                    };

                    Self::ConfigFilePath(src.to_string())
                } else if arg.starts_with("--gen-config=") {
                    let dst = arg.split("=").collect::<Vec<_>>();
                    let dst = if dst.len() == 1 {
                        ""
                    } else {
                        dst.last().unwrap_or(&"")
                    };

                    Self::GenerateConfig(dst.to_string())
                } else {
                    Self::Unknown(arg.to_string())
                };

                vec![ret]
            }
        }
    }
}

impl Default for AppOptions {
    fn default() -> Self {
        Self::Run
    }
}

impl Display for AppOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::Run | Self::ConfigFilePath(_) | Self::GenerateConfig(_) => String::new(),
            Self::Version => {
                let app_name = env!("CARGO_PKG_NAME");
                let app_version = env!("CARGO_PKG_VERSION");

                format!("{} {}", app_name, app_version)
            }
            Self::Help => {
                let lines = [
                    "Usage: needle [OPTIONS]",
                    "",
                    "Options:",
                    "   -h, --help                  Display this message",
                    "   -c, --config [FILENAME]     Specify a path to a custom config file",
                    "                               If path of config file is not specified, it will default to default path",
                    "       --gen-config [FILENAME] Generates config file",
                    "                               If path is specified, config file is generated to the specified path",
                    "                               Default path:",
                    "                               - Linux: $HOME/.config/needle/config.toml",
                    "                               - Windows: %AppData%\\Roaming\\bonohub13\\needle\\config\\config.toml",
                    "   -v, --version           Print version info and exit",
                ];

                lines
                    .iter()
                    .map(|s| format!("{}{}", s, Self::NEWLINE))
                    .collect()
            }
            Self::Unknown(err) => {
                let err = err
                    .strip_prefix("--")
                    .unwrap_or(err)
                    .strip_prefix("-")
                    .unwrap_or(err);

                format!("Unknown option: '{}'{}", err, Self::NEWLINE)
            }
        };

        write!(f, "{}", msg)
    }
}
