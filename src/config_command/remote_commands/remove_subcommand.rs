use rdm_macros::{FromError, ToDoc};

use crate::config::Config;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "remove error:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum RemoveError {
    #[doc_format(format_str = "Remote {} does not exists.", _1)]
    DoesNotExists(String),
    #[doc_to_string]
    GitError(git2::Error),
}

pub(super) fn run(config: Config, name: String) -> Result<(), RemoveError> {
    let repo = config.repo;

    if repo.find_remote(name.as_str()).is_err() {
        return Err(RemoveError::DoesNotExists(name));
    }

    repo.remote_delete(name.as_str())?;
    log::info!("Remote {} was deleted", name);

    Ok(())
}
