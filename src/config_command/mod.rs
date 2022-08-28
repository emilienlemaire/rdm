mod add_subcommand;
mod save_subcommand;
mod status_subcommand;
mod update_subcommand;

use rdm_macros::{FromError, ToDoc};

use crate::{args::ConfigSubCommand, config::Config};

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Config Command Error:"]
pub(crate) enum ConfigCommandError {
    AddSubcommandError(add_subcommand::AddSubcommandError),
    UpdateSubcommandError(update_subcommand::UpdateSubcommandError),
    StatusSubcommandError(status_subcommand::StatusSubcommandError),
    SaveSubcommandError(save_subcommand::SaveSubcommandError),
}

pub(crate) fn run(
    sub_command: ConfigSubCommand,
    config: Config,
) -> Result<(), ConfigCommandError> {
    match sub_command {
        ConfigSubCommand::Add { path } => add_subcommand::run(config, path)?,
        ConfigSubCommand::Update { path } => {
            update_subcommand::run(config, path)?
        }
        ConfigSubCommand::Status { untracked } => {
            status_subcommand::run(config, untracked)?
        }
        ConfigSubCommand::Save => save_subcommand::run(config)?,
    };

    Ok(())
}
