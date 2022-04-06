use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(description = "A tool to help with testing, and dev-related tasks")]
pub struct Args {
    #[argh(switch, description = "show the executable version")]
    pub version: bool,

    #[argh(subcommand)]
    pub nested: Option<Subcommands>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Subcommands {
    Ci(CiArgs),
    Autocomplete(AutocompleteArgs),
    Config(ConfigArgs),
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

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "config",
    description = "interract with configuration"
)]
pub struct ConfigArgs {
    #[argh(subcommand)]
    pub command: ConfigSubcommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum ConfigSubcommands {
    Migrate(MigrateArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "migrate",
    description = "migrate to and from config"
)]
pub struct MigrateArgs {
    #[argh(subcommand)]
    pub to: MigrateToSubCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum MigrateToSubCommands {
    V0y(V0yArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "unstable",
    description = "migrate config to unstable"
)]
pub struct V0yArgs {}
