use std::{
    str::FromStr,
    time::Duration,
};

use anyhow::Result;
use chrono::{
    DateTime,
    FixedOffset,
    Utc,
};
use clap::Arg;

#[derive(Debug, Eq, PartialEq)]
pub enum Privilege {
    Normal,
    Experimental,
}

#[derive(Debug)]
pub struct CallArgs {
    pub privileges: Privilege,
    pub command: Command,
}

impl CallArgs {
    pub fn validate(&self) -> Result<()> {
        if self.privileges == Privilege::Experimental {
            return Ok(());
        }

        match &self.command {
            // | Command::Experimental { .. } => Err(Error::ExperimentalCommand("watch".to_owned()))?,
            | _ => (),
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ManualFormat {
    Manpages,
    Markdown,
}

#[derive(Debug)]
pub enum Command {
    Manual {
        path: String,
        format: ManualFormat,
    },
    Autocomplete {
        path: String,
        shell: clap_complete::Shell,
    },

    Run {
        until: Option<DateTime<FixedOffset>>,
        interval: Duration,
    },
}

pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    pub fn root_command() -> clap::Command {
        clap::Command::new("iambusy")
            .version(env!("CARGO_PKG_VERSION"))
            .about("iambusy - i am busy")
            .author("Alexander Weber <aw@voidpointergroup.com>")
            .propagate_version(true)
            .subcommand_required(true)
            .args([Arg::new("experimental")
                .short('e')
                .long("experimental")
                .help("Enables experimental features.")
                .num_args(0)])
            .subcommand(
                clap::Command::new("man")
                    .about("Renders the manual.")
                    .arg(clap::Arg::new("out").short('o').long("out").required(true))
                    .arg(
                        clap::Arg::new("format")
                            .short('f')
                            .long("format")
                            .value_parser(["manpages", "markdown"])
                            .required(true),
                    ),
            )
            .subcommand(
                clap::Command::new("autocomplete")
                    .about("Renders shell completion scripts.")
                    .arg(clap::Arg::new("out").short('o').long("out").required(true))
                    .arg(
                        clap::Arg::new("shell")
                            .short('s')
                            .long("shell")
                            .value_parser(["bash", "zsh", "fish", "elvish", "powershell"])
                            .required(true),
                    ),
            )
            .subcommand(
                clap::Command::new("run")
                    .about("Run the program.")
                    .arg(clap::Arg::new("until").long("until").required(false).conflicts_with("for"))
                    .arg(clap::Arg::new("for").long("for").required(false).conflicts_with("until"))
                    .arg(clap::Arg::new("interval").short('i').long("interval").required(false).default_value("1s")),
            )
    }

    pub fn load() -> Result<CallArgs> {
        let command = Self::root_command().get_matches();

        let privileges = if command.get_flag("experimental") {
            Privilege::Experimental
        } else {
            Privilege::Normal
        };

        let cmd = if let Some(subc) = command.subcommand_matches("man") {
            Command::Manual {
                path: subc.get_one::<String>("out").unwrap().into(),
                format: match subc.get_one::<String>("format").unwrap().as_str() {
                    | "manpages" => ManualFormat::Manpages,
                    | "markdown" => ManualFormat::Markdown,
                    | _ => return Err(anyhow::anyhow!("unknown format")),
                },
            }
        } else if let Some(subc) = command.subcommand_matches("autocomplete") {
            Command::Autocomplete {
                path: subc.get_one::<String>("out").unwrap().into(),
                shell: clap_complete::Shell::from_str(subc.get_one::<String>("shell").unwrap().as_str()).unwrap(),
            }
        } else if let Some(subc) = command.subcommand_matches("run") {
            let until = if let Some(v) = subc.get_one::<String>("until") {
                Some(DateTime::parse_from_rfc3339(v.as_str()).unwrap())
            } else if let Some(v) = subc.get_one::<String>("for") {
                Some((Utc::now() + parse_duration::parse(v)?).fixed_offset())
            } else {
                None
            };

            Command::Run {
                until,
                interval: parse_duration::parse(subc.get_one::<String>("interval").unwrap())?,
            }
        } else {
            return Err(anyhow::anyhow!("unknown command"));
        };

        let callargs = CallArgs {
            privileges,
            command: cmd,
        };

        callargs.validate()?;
        Ok(callargs)
    }
}

mod test {
    #[tokio::test]
    async fn test_args() -> anyhow::Result<()> {
        chrono::DateTime::parse_from_rfc3339("2024-08-15T14:15:00.000-07:00").unwrap();
        Ok(())
    }
}
