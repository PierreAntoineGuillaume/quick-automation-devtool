mod ci;
mod config;

extern crate atty;
extern crate indexmap;
extern crate term;
extern crate terminal_size;
extern crate textwrap;

use crate::ci::display::TermCiDisplay;
use crate::ci::Ci;
use crate::config::argh::{Args, ConfigSubcommands, MigrateToSubCommands, Subcommands};
use crate::config::migrate::{Migrate, Migration};
use crate::config::{Config, ConfigPayload, OptionConfigPayload};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Args = argh::from_env();

    let mut option_config = OptionConfigPayload::default();

    args.fill(&mut option_config);

    if args.version {
        println!("v{}", VERSION);
        std::process::exit(0);
    }

    let command = args.nested.unwrap_or_else(|| {
        println!("No args given");
        std::process::exit(0);
    });

    if matches!(command, Subcommands::Autocomplete(_)) {}

    let envvar = std::env::var("DT_CONFIG_FILE")
        .or_else::<String, _>(|_| Ok(String::from("dt")))
        .unwrap();

    let config = Config::from(option_config, &envvar);

    match command {
        Subcommands::Ci(_) => match (Ci {}).run(config) {
            Ok(_) => {}
            Err(str) => {
                if let Some(msg) = str {
                    eprintln!("{}", msg);
                }
                std::process::exit(1)
            }
        },
        Subcommands::Autocomplete(_) => {
            if atty::is(atty::Stream::Stdout) {
                eprintln!(
                    "#{} autocomplete > ~/.local/share/bash-completion/completions/{}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_NAME")
                );
            }
            print!("{}", include_str!("../assets/dt_bash_competion.sh"));
            std::process::exit(0);
        }
        Subcommands::Config(config_args) => match config_args.command {
            ConfigSubcommands::Migrate(version) => {
                let mut payload = ConfigPayload::default();
                if let Err(err) = config.load_into(&mut payload) {
                    eprintln!("dt: could not read config: {}", err);
                    std::process::exit(1);
                } else {
                    let migrate = Migrate::new(config);
                    match match version.to {
                        MigrateToSubCommands::V0y(_) => migrate.to0y(),
                        MigrateToSubCommands::V0x(_) => migrate.to0x(),
                    } {
                        Ok(serializable) => match serializable {
                            Migration::Version0x(version) => {
                                println!("{}", toml::to_string(&version).unwrap())
                            }
                            Migration::Version0y(version) => {
                                println!("{}", toml::to_string(&version).unwrap())
                            }
                        },
                        Err(message) => eprintln!("dt: {}", message),
                    }
                }
            }
        },
    }
}
