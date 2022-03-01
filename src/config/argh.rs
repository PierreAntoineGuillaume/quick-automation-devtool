use crate::OptionConfigPayload;
use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(description = "dt is a tool to help with testing, and dev-related tasks")]
pub struct Args {
    #[argh(switch, description = "show the executable version")]
    pub version: bool,
    #[argh(switch, short = 'q', description = "do not display expected output")]
    pub quiet: bool,

    #[argh(subcommand)]
    pub nested: Option<Subcommands>,
}

impl Args {
    pub fn fill(&self, config: &mut OptionConfigPayload) {
        config.display = if self.quiet { Some(()) } else { None }
    }
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Subcommands {
    Ci(CiArgs),
    Autocomplete(AutocompleteArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ci", description = "play the ci")]
pub struct CiArgs {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "autocomplete",
    description = "generate bash completion script"
)]
pub struct AutocompleteArgs {}
