mod ci;
mod config;

extern crate atty;
extern crate term;

use crate::ci::display::TermCiDisplay;
use crate::ci::job::Pipeline;
use crate::ci::schedule::CompositeJobScheduler;
use crate::ci::ParrallelJobStarter;
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

    match command {
        Subcommands::Ci(_) => {
            let mut pipeline = Pipeline::new();
            let envvar = std::env::var("DT_CONFIG_FILE")
                .or_else::<String, _>(|_| Ok(String::from("dt.toml")))
                .unwrap();

            let config = Config::parse(&envvar)
                .map_err(|error| {
                    eprintln!("dt: {}", error);
                    std::process::exit(1);
                })
                .unwrap();

            config.load_into(&mut pipeline);

            if pipeline
                .run(&mut CompositeJobScheduler::<
                    ParrallelJobStarter,
                    TermCiDisplay,
                >::new(
                    &mut ParrallelJobStarter::new(),
                    &mut TermCiDisplay::new(),
                ))
                .is_err()
            {
                std::process::exit(1);
            }
        }
        Subcommands::Autocomplete(_) => {
            if atty::is(atty::Stream::Stdout) {
                eprintln!(
                    "#{} autocomplete > ~/.local/share/bash-completion/completions/{}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_NAME")
                );
            }
            print!("{}", include_str!("../assets/dt_bash_competion.sh"));
        }
    }
}
