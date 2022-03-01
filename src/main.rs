mod ci;
mod config;

extern crate atty;
extern crate indexmap;
extern crate term;
extern crate terminal_size;

use crate::ci::display::TermCiDisplay;
use crate::ci::Ci;
use crate::config::argh::{Args, Subcommands};
use crate::config::{Config, OptionConfigPayload};

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

    if matches!(command, Subcommands::Autocomplete(_)) {
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

    let envvar = std::env::var("DT_CONFIG_FILE")
        .or_else::<String, _>(|_| Ok(String::from("dt")))
        .unwrap();

    let config = Config::from(option_config, &envvar);

    if let Subcommands::Ci(_) = command {
        match (Ci {}).run(config) {
            Ok(_) => {}
            Err(str) => {
                if let Some(msg) = str {
                    eprintln!("{}", msg);
                }
                std::process::exit(1)
            }
        }
    }
}
