mod ci;
mod config;

extern crate atty;
extern crate indexmap;
extern crate term;

use crate::ci::display::TermCiDisplay;
use crate::ci::Ci;
use crate::config::Config;
use argh::FromArgs;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(FromArgs, PartialEq, Debug)]
#[argh(description = "dt is a tool to help with testing, and dev-related tasks")]
struct Args {
    #[argh(switch, short = 'v', description = "show the executable version")]
    version: bool,

    #[argh(subcommand)]
    nested: Option<Subcommands>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Subcommands {
    Ci(CiArgs),
    Autocomplete(AutocompleteArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ci", description = "play the ci")]
struct CiArgs {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "autocomplete",
    description = "generate bash completion script"
)]
struct AutocompleteArgs {}

fn main() {
    let args: Args = argh::from_env();

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

    let config = Config::from(&envvar);

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
