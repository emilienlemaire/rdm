use git2::{Status, StatusOptions};
use rdm_macros::{FromError, ToDoc};

use crate::{
    config::Config,
    lockfile::{self, increment_revision},
};

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while running the `save' subcommand:"]
pub(crate) enum SaveSubcommandError {
    #[doc_text = "No changes were saved to the config, try to run `rdm config update'."]
    NoChangesError,
    LockFileError(lockfile::LockFileError),
    #[doc_to_string]
    GitError(git2::Error),
}

pub(super) fn run(config: Config) -> Result<(), SaveSubcommandError> {
    let repo = &config.repo;
    let revision = config.revision;

    let mut status_opts = StatusOptions::new();
    status_opts.include_ignored(false);
    status_opts.include_unmodified(false);
    status_opts.include_untracked(false);

    let statuses_before = repo.statuses(Some(&mut status_opts))?;

    if statuses_before.is_empty() {
        return Err(SaveSubcommandError::NoChangesError);
    }

    increment_revision(&config)?;

    let mut index = repo.index()?;
    let oid = index.write_tree()?;
    let sig = repo.signature()?;
    let parent = repo.head()?;
    let parent = parent.peel_to_commit()?;
    let tree = repo.find_tree(oid)?;

    let msg = format!("Revision #{}", revision);

    repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&parent])?;

    log::info!(
        "The revision #{} of your config was saved with the following changes:",
        revision
    );
    statuses_before.iter().for_each(|e| match e.status() {
        Status::INDEX_NEW => rdm_log::StatusLogger::new_file(e.path().unwrap()),
        Status::INDEX_MODIFIED => {
            rdm_log::StatusLogger::modified_file(e.path().unwrap())
        }
        Status::INDEX_DELETED => {
            rdm_log::StatusLogger::removed_file(e.path().unwrap())
        }
        _ => (),
    });

    Ok(())
}
