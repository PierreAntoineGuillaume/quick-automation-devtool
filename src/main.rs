#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![allow(clippy::use_self)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::default_trait_access)]

pub mod app;
mod ci;
mod config;

use std::io::Write;
use std::sync::mpsc::channel;

use crate::app::domain::{Event, State};
use crate::ci::config::CliOption;
use crate::ci::Ci;
use crate::config::argh::{Args, CiArgs, Subcommands};
use crate::config::Config;

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

    let no_tty = args.no_tty;

    let envvar = std::env::var(format!("{}_CONFIG_FILE", PACKAGE_NAME.to_uppercase()))
        .or_else::<String, _>(|_| Ok(String::from(PACKAGE_NAME)))
        .unwrap();

    let config = if let Some(name) = args.file {
        Config::from_name(&name)
    } else {
        Config::from(&envvar)
    };

    match command {
        Subcommands::Autocomplete(args) => {
            const DIR: &str = "~/.local/share/bash-completion/completions";

            if args.apply {
                let home = std::env::var("HOME")
                    .unwrap_or_else(|e| panic!("Failure reading env var HOME: {e}"));
                let dir = format!("{home}/.local/share/bash-completion/completions");
                let file = format!("{dir}/{PACKAGE_NAME}");
                if !std::path::Path::new(&dir).is_dir() {
                    println!("Creating folder {dir}");
                    std::fs::create_dir_all(DIR)
                        .unwrap_or_else(|e| panic!("Failure creating {dir}:\n{e}"));
                }
                println!("Writing file {file}");
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(&file)
                    .unwrap_or_else(|e| panic!("Failed opening `{file}`: {e}"));
                f.write_all(include_str!("../assets/bash_completion.sh").as_ref())
                    .unwrap_or_else(|e| panic!("Failed writing to `{file}`: {e}"));
                f.flush()
                    .unwrap_or_else(|e| panic!("Failed flushing `{file}`: {e}"));
                return;
            }
            let file = format!("{DIR}/{PACKAGE_NAME}");

            if atty::is(atty::Stream::Stdout) {
                eprintln!(
                    "# Some text is appended to the output to explain how to use this command."
                );
                eprintln!("# Skip to the end to get easy instructions.");
                eprintln!("# If you wish to copy the content, just pipe the result of the command in cat:");
                eprintln!("# {PACKAGE_NAME} autocomplete | cat");
                eprintln!("# It will get rid of this extraneous output.");
                eprintln!("# The autocompletion script starts here:\n");
            }
            println!("{}", include_str!("../assets/bash_completion.sh"));
            if atty::is(atty::Stream::Stdout) {
                eprintln!("# To register {PACKAGE_NAME}'s bash autocompletion script");
                eprintln!("# {file}:");
                eprintln!("# mkdir -p {DIR}");
                eprintln!("# {PACKAGE_NAME} autocomplete > {file}",);
            }
            std::process::exit(0);
        }
        Subcommands::App(_) => app(),
        Subcommands::Ci(arg) => ci_run(&config, arg, no_tty),
        Subcommands::Debug(arg) => match Ci::debug(&config, arg.nested) {
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
        Subcommands::HasCi(_) => {
            if config.get_first_available_config_file().is_err() {
                std::process::exit(1);
            }
        }
    }
}

fn ci_run(config: &Config, arg: CiArgs, no_tty: bool) {
    match Ci::run(
        config,
        &CliOption {
            job: arg.nested,
            no_tty,
        },
    ) {
        Ok(true) => {}
        Ok(false) => {
            std::process::exit(1);
        }
        Err(str) => {
            eprintln!("{PACKAGE_NAME}: {str}");
            std::process::exit(2)
        }
    }
}

fn app() {
    eprintln!("{PACKAGE_NAME}: app is an experimental feature");
    eprintln!("it may be removed or reworked in the future and is unstable");
    let (tx, _rx) = channel();
    let consumer = app::infra::Fake { stream: tx };
    let state = State::default();

    if let Err(e) = app::domain::run(&consumer, state, &Event::Awaiting) {
        eprintln!("{PACKAGE_NAME} error: {e}");
    }
}

#[macro_export]
macro_rules! strvec {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}
