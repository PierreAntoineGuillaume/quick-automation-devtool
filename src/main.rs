mod ci;

extern crate atty;

use crate::ci::display::OneOffCiDisplay;
use crate::ci::job::Pipeline;
use crate::ci::schedule::CompositeJobScheduler;
use crate::ci::ParrallelJobStarter;
use argh::FromArgs;

const VERSION: &str = "0.1.2";

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

            pipeline.push("phpstan", vec!["vendor/bin/phpstan"]);
            pipeline.push("phpcs", vec!["vendor/bin/phpcs"]);
            pipeline.push("tests", vec!["yarn install", "yarn jest"]);

            if pipeline
                .run(&mut CompositeJobScheduler::new(
                    &mut ParrallelJobStarter::new(),
                    &mut OneOffCiDisplay::new(),
                ))
                .is_err()
            {
                std::process::exit(1);
            }
        }
        Subcommands::Autocomplete(_) => {
            if atty::is(atty::Stream::Stdout) {
                eprintln!("#dt autocomplete > ~/.local/share/bash-completion/completions/dt");
            }
            print!("{}", include_str!("../assets/dt_bash_competion.sh"));
        }
    }
}
