#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::unused_self)]

mod ci;
mod config;

extern crate anyhow;
extern crate atty;
extern crate crossterm;
extern crate indexmap;
extern crate terminal_size;
extern crate tui;

use crate::ci::config::CliOption;
use crate::ci::Ci;
use crate::config::argh::{Args, ConfigSubcommands, MigrateToSubCommands, Subcommands};
use crate::config::migrate::Migrate;
use crate::config::{Config, Format, Payload};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    let args: Args = argh::from_env();

    if args.version {
        println!("v{}", VERSION);
        std::process::exit(0);
    }

    let command = args.nested.unwrap_or_else(|| {
        println!("{PACKAGE_NAME}: no args given");
        std::process::exit(0);
    });

    if matches!(command, Subcommands::Autocomplete(_)) {}

    let envvar = std::env::var(format!("{}_CONFIG_FILE", PACKAGE_NAME.to_uppercase()))
        .or_else::<String, _>(|_| Ok(String::from(PACKAGE_NAME)))
        .unwrap();

    let config = Config::from(&envvar);

    match command {
        Subcommands::Ci(arg) => match (Ci {}).run(&config, &CliOption { job: arg.nested }) {
            Ok(true) => {}
            Ok(false) => {
                std::process::exit(1);
            }
            Err(str) => {
                eprintln!("{PACKAGE_NAME}: {}", str);
                std::process::exit(2)
            }
        },
        Subcommands::List(_) => match (Ci {}).list(&config) {
            Ok(()) => {}
            Err(str) => {
                eprintln!("{PACKAGE_NAME}: {}", str);
                std::process::exit(2)
            }
        },
        Subcommands::Autocomplete(_) => {
            if atty::is(atty::Stream::Stdout) {
                const DIR: &str = "~/.local/share/bash-completion/completions";
                eprintln!("# To register {PACKAGE_NAME}'s bash autocompletion script");
                eprintln!("# put the following content including the shebang (#!/bin/bash) in");
                eprintln!("# {DIR}/{PACKAGE_NAME}:");
                eprintln!("# mkdir -p {}", DIR);
                eprintln!("# {PACKAGE_NAME} autocomplete > {DIR}/{PACKAGE_NAME}",);
            }
            print!("{}", include_str!("../assets/bash_completion.sh"));
            std::process::exit(0);
        }
        Subcommands::Config(config_args) => match config_args.command {
            ConfigSubcommands::Migrate(version) => {
                let mut payload = Payload::default();
                if let Err(err) = config.load_into(&mut payload) {
                    eprintln!("{PACKAGE_NAME}: could not read config: {}", err);
                    std::process::exit(1);
                } else {
                    let used_conf_file = config
                        .get_first_available_config_file()
                        .expect("Error passed before");
                    let format = config
                        .get_parser(&used_conf_file)
                        .expect("Error passed before")
                        .format();

                    let migrate = Migrate::new(config);
                    let migration = match version.to {
                        MigrateToSubCommands::V1(_) => migrate.to0y(),
                    };
                    if migration.is_err() {
                        eprintln!("{PACKAGE_NAME}: {}", migration.unwrap_err());
                        std::process::exit(1)
                    }

                    let serialization = match (format, migration.unwrap()) {
                        (Format::Yaml, serializable) => migrate.yaml(serializable),
                    };
                    println!("{}", serialization);
                }
            }
        },
    }
}

#[macro_export]
macro_rules! strvec {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}
