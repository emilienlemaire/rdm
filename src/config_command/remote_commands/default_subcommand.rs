use rdm_macros::{FromError, ToDoc};

use crate::config::Config;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "default error:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum DefaultError {
    #[doc_to_string]
    GitError(git2::Error),
    #[doc_text = "HEAD is not on a branch."]
    HeadNoBranch,
    #[doc_format(format_str = "The remote {} was not found.", _1)]
    #[no_from]
    RemoteNotFound(String),
}

pub(super) fn run(config: Config, name: String) -> Result<(), DefaultError> {
    let repo = config.repo;

    let head = repo.head()?;
    if repo.find_remote(&name).is_err() {
        return Err(DefaultError::RemoteNotFound(name));
    }

    if head.is_branch() {
        let mut config = repo.config()?;
        let refs = head.name().unwrap().to_string();
        let head_name = head.shorthand().unwrap().to_string();

        let remote_key = format!("branch.{}.remote", head_name);
        let merge_key = format!("branch.{}.merge", head_name);

        config.set_str(&remote_key, &name)?;
        log::info!("Set the remote for {} to {}", head_name, name);

        config.set_str(&merge_key, &refs)?;
        log::info!("Set the merge ref for {} to {}", head_name, refs);
    } else {
        return Err(DefaultError::HeadNoBranch);
    }

    Ok(())
}
