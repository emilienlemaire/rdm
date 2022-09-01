use rdm_macros::{FromError, ToDoc};

use crate::{
    config::Config, config_command::remote_commands::default_subcommand,
};

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "add error:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum RemoteAddError {
    #[doc_to_string]
    GitError(git2::Error),
    #[doc_format(format_str = "Remote {} already exists.", _1)]
    AlreadyExists(String),
    DefaultError(default_subcommand::DefaultError),
}

pub(super) fn run(
    config: Config,
    name: String,
    url: String,
    default: bool,
) -> Result<(), RemoteAddError> {
    let repo = &config.repo;

    match repo.find_remote(name.as_str()) {
        Err(_) => (),
        Ok(_) => return Err(RemoteAddError::AlreadyExists(name)),
    };

    repo.remote(&name, &url)?;
    log::info!("Remote `{}' was added with url: {}", name, url);

    if default {
        default_subcommand::run(config, name)?;
    }

    Ok(())
}
