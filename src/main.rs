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
use crate::config::migrate::Migrate;
use crate::config::{Config, ConfigPayload, Format, OptionConfigPayload};

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
        println!("dt: no args given");
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
                    eprintln!("dt: {}", msg);
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
                    let used_conf_file = config
                        .get_first_available_config_file()
                        .expect("Error passed before");
                    let mut format = config
                        .get_parser(&used_conf_file)
                        .expect("Error passed before")
                        .format();

                    let migrate = Migrate::new(config);
                    let migration = match version.to {
                        MigrateToSubCommands::V0y(sub) => {
                            if let Some(new_format) = sub.format {
                                format = new_format.map()
                            }
                            migrate.to0y()
                        }
                        MigrateToSubCommands::V0x(sub) => {
                            if let Some(new_format) = sub.format {
                                format = new_format.map()
                            }
                            migrate.to0x()
                        }
                    };
                    if migration.is_err() {
                        eprintln!("dt: {}", migration.unwrap_err());
                        std::process::exit(1)
                    }

                    let serialization = match (format, migration.unwrap()) {
                        (Format::Yaml, serializable) => migrate.yaml(serializable),
                        (Format::Toml, serializable) => migrate.toml(serializable),
                    };
                    println!("{}", serialization)
                }
            }
        },
    }
}
