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
    Config(ConfigArgs),
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
pub struct AutocompleteArgs {}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "config",
    description = "interract with configuration"
)]
pub struct ConfigArgs {
    #[argh(subcommand)]
    pub command: ConfigSubcommands,
}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(subcommand)]
pub enum ConfigSubcommands {
    Migrate(MigrateArgs),
}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "migrate",
    description = "migrate to and from config"
)]
pub struct MigrateArgs {
    #[argh(subcommand)]
    pub to: MigrateToSubCommands,
}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(subcommand)]
pub enum MigrateToSubCommands {
    V1(V1Args),
}

#[derive(FromArgs, Eq, PartialEq, Debug)]
#[argh(subcommand, name = "1", description = "migrate config to version 1")]
pub struct V1Args {}

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
