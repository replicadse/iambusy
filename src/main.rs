mod args;
mod reference;

use std::path::PathBuf;

use anyhow::Result;
use args::ManualFormat;
use chrono::Utc;
use enigo::Keyboard;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = crate::args::ClapArgumentLoader::load()?;

    match cmd.command {
        | crate::args::Command::Manual { path, format } => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path)?;
            match format {
                | ManualFormat::Manpages => {
                    reference::build_manpages(&out_path)?;
                },
                | ManualFormat::Markdown => {
                    reference::build_markdown(&out_path)?;
                },
            }
            Ok(())
        },
        | crate::args::Command::Autocomplete { path, shell } => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path)?;
            reference::build_shell_completion(&out_path, &shell)?;
            Ok(())
        },
        | crate::args::Command::Run { until, interval, type_ } => {
            let mut e = enigo::Enigo::new(&enigo::Settings::default())?;
            loop {
                if let Some(until) = until {
                    if Utc::now() > until {
                        break;
                    }
                }
                e.text(&type_)?;
                std::thread::sleep(interval);
            }

            Ok(())
        },
    }
}
