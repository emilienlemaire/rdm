use rdm_macros::{FromError, ToDoc};

use crate::{args::RemoteSubCommand, config::Config};

mod add_subcommand;
mod list_subcommand;
mod remove_subcommand;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Remote"]
pub(crate) enum RemoteError {
    AddError(add_subcommand::RemoteAddError),
    RemoveError(remove_subcommand::RemoveError),
    ListError(list_subcommand::ListError),
}

pub(super) fn run(
    config: Config,
    sub_command: RemoteSubCommand,
) -> Result<(), RemoteError> {
    match sub_command {
        RemoteSubCommand::Add { name, url } => {
            add_subcommand::run(config, name, url)?
        }
        RemoteSubCommand::Remove { name } => {
            remove_subcommand::run(config, name)?
        }
        RemoteSubCommand::List => list_subcommand::run(config)?,
    };
    Ok(())
}
