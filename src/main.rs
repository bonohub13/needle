#![windows_subsystem = "windows"]
mod app;
mod needle;
mod options;

use app::*;
use needle::*;
use options::*;

use anyhow::{bail, Result};
use needle_core::NeedleConfig;
use std::sync::Arc;
#[cfg(target_os = "windows")]
use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};

fn main() -> Result<()> {
    // Enable CLI for Windows (A workaround for #![windows_subsystem = "windows"])
    //  Source: https://github.com/rust-lang/rust/issues/67159#issuecomment-987882771 (by phiresky)
    #[cfg(target_os = "windows")]
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    env_logger::init();

    let app_option = AppOptions::new();
    let mut config_path = None;

    for opt in app_option.iter() {
        match opt {
            AppOptions::Help | AppOptions::Version => {
                println!("{}", opt);

                return Ok(());
            }
            AppOptions::GenerateConfig(path) => {
                return NeedleConfig::config(Some(path));
            }
            AppOptions::Unknown(_) => bail!("{}", opt),
            AppOptions::ConfigFilePath(path) => {
                config_path = Some(path.as_str());
            }
            _ => (),
        }
    }

    let config = Arc::new(NeedleConfig::from(config_path)?);

    run(config)
}
