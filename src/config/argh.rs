use crate::{Format, OptionConfigPayload};
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
    V0x(V0xArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum MigrateFormatSubCommand {
    Toml(TomlArgs),
    Yaml(YamlArgs),
}

impl MigrateFormatSubCommand {
    pub fn map(&self) -> Format {
        match self {
            MigrateFormatSubCommand::Toml(_) => Format::Toml,
            MigrateFormatSubCommand::Yaml(_) => Format::Yaml,
        }
    }
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "unstable",
    description = "migrate config to unstable"
)]
pub struct V0yArgs {
    #[argh(subcommand)]
    pub format: Option<MigrateFormatSubCommand>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "0.x", description = "migrate config to 0.x")]
pub struct V0xArgs {
    #[argh(subcommand)]
    pub format: Option<MigrateFormatSubCommand>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "to-toml", description = "migrate config to toml")]
pub struct TomlArgs {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "to-yaml", description = "migrate config to yaml")]
pub struct YamlArgs {}
