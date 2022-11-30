#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![allow(clippy::use_self)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::default_trait_access)]

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
use crate::config::{Config, Payload};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    let args: Args = argh::from_env();

    if args.version {
        println!("v{VERSION}");
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

    let config = if let Some(name) = args.file {
        Config::from_name(&name)
    } else {
        Config::from(&envvar)
    };

    match command {
        Subcommands::App(_) => {
            eprintln!("{PACKAGE_NAME}: app is an experimental feature");
            if cfg!(feature = "app") {
                eprintln!("it may be removed or reworked in the future and is unstable");
            } else {
                eprintln!("try compiling {PACKAGE_NAME} with the `app` feature enabled");
            }
        }
        Subcommands::Ci(arg) => match Ci::run(&config, &CliOption { job: arg.nested }) {
            Ok(true) => {}
            Ok(false) => {
                std::process::exit(1);
            }
            Err(str) => {
                eprintln!("{PACKAGE_NAME}: {str}");
                std::process::exit(2)
            }
        },
        Subcommands::List(_) => match Ci::list(&config) {
            Ok(()) => {}
            Err(str) => {
                eprintln!("{PACKAGE_NAME}: {str}");
                std::process::exit(2)
            }
        },
        Subcommands::Autocomplete(_) => {
            if atty::is(atty::Stream::Stdout) {
                const DIR: &str = "~/.local/share/bash-completion/completions";
                eprintln!("# To register {PACKAGE_NAME}'s bash autocompletion script");
                eprintln!("# put the following content including the shebang (#!/bin/bash) in");
                eprintln!("# {DIR}/{PACKAGE_NAME}:");
                eprintln!("# mkdir -p {DIR}");
                eprintln!("# {PACKAGE_NAME} autocomplete > {DIR}/{PACKAGE_NAME}",);
            }
            print!("{}", include_str!("../assets/bash_completion.sh"));
            std::process::exit(0);
        }
        Subcommands::Config(config_args) => match config_args.command {
            ConfigSubcommands::Migrate(version) => {
                let mut payload = Payload::default();
                if let Err(err) = config.load_into(&mut payload) {
                    eprintln!("{PACKAGE_NAME}: could not read config: {err}");
                    std::process::exit(1);
                } else {
                    let migrate = Migrate::new(config);
                    let migration = match version.to {
                        MigrateToSubCommands::V1(_) => migrate.to1(),
                    };
                    if migration.is_err() {
                        eprintln!("{PACKAGE_NAME}: {}", migration.unwrap_err());
                        std::process::exit(1);
                    }

                    let serialization = Migrate::yaml(migration.unwrap());
                    println!("{serialization}");
                }
            }
        },
        Subcommands::HasCi(_) => {
            if config.get_first_available_config_file().is_err() {
                std::process::exit(1);
            }
        }
    }
}

#[macro_export]
macro_rules! strvec {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}
