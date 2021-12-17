mod ci;

use crate::ci::logic::job::{JobScheduler, Pipeline};
use crate::ci::NThreadedJobScheduler;
use argh::FromArgs;

const VERSION: &str = "0.1";

#[derive(FromArgs, PartialEq, Debug)]
#[argh(description = "pcg is a tool to help with testing, and dev-related tasks")]
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
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ci", description = "play the ci")]
struct CiArgs {}

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

            pipeline.push("phpstan".into(), "vendor/bin/phpstan".into());
            pipeline.push("phpcs".into(), "vendor/bin/phpcs".into());

            let mut scheduler: Box<dyn JobScheduler> = Box::new(NThreadedJobScheduler {});
            pipeline.run(&mut (*scheduler))
        }
    }
}
