use argh::FromArgs;

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(description = "A tool to help with testing, and dev-related tasks")]
pub struct Args {
    #[argh(switch, description = "show the executable version")]
    pub version: bool,

    #[argh(switch, description = "override the config to allow tty-less ci")]
    pub no_tty: bool,

    #[argh(subcommand)]
    pub nested: Option<Subcommands>,

    #[argh(
        option,
        short = 'f',
        description = "specify an alternate qad config file"
    )]
    pub file: Option<String>,
}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Subcommands {
    Ci(CiArgs),
    List(ListArgs),
    Autocomplete(AutocompleteArgs),
    HasCi(HasCiArgs),
    App(AppArgs),
    Debug(DebugArgs),
}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(subcommand, name = "ci", description = "play the ci")]
pub struct CiArgs {
    #[argh(positional, description = "an optionnal job or group to run")]
    pub nested: Option<String>,
}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(subcommand, name = "list", description = "list jobs")]
pub struct ListArgs {}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "autocomplete",
    description = "generate bash completion script"
)]
pub struct AutocompleteArgs {
    #[argh(switch, description = "apply the modifications")]
    pub apply: bool,
}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "has-ci",
    description = "checks whether current folder has qad file- or fails"
)]
pub struct HasCiArgs {}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(subcommand, name = "app", description = "starts a ci app")]
pub struct AppArgs {}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(subcommand, name = "debug", description = "debugs a ci job")]
pub struct DebugArgs {
    #[argh(positional, description = "job name")]
    pub nested: String,
}
