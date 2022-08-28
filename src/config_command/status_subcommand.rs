use git2::{Status, StatusOptions};
use rdm_macros::{FromError, ToDoc};

use crate::config::Config;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while running the `status' subcommand:"]
pub(crate) enum StatusSubcommandError {
    #[doc_to_string]
    GitError(git2::Error),
}

pub(super) fn run(
    config: Config,
    show_untracked: bool,
) -> Result<(), StatusSubcommandError> {
    let repo = config.repo;
    let mut status_opts = StatusOptions::new();
    status_opts.include_unmodified(false);
    status_opts.include_untracked(show_untracked);
    status_opts.include_ignored(false);

    let status = repo.statuses(Some(&mut status_opts))?;

    if !status.is_empty() {
        println!("Current status of your configuration:");
        for entry in status.iter() {
            match entry.status() {
                Status::CURRENT => {}
                Status::WT_NEW => {
                    if show_untracked {
                        rdm_log::StatusLogger::untracked_file(
                            entry.path().unwrap(),
                        )
                    }
                }
                Status::INDEX_NEW => {
                    rdm_log::StatusLogger::new_file(entry.path().unwrap())
                }
                Status::WT_MODIFIED => {
                    rdm_log::StatusLogger::modified_unsaved_file(
                        entry.path().unwrap(),
                    )
                }
                Status::INDEX_MODIFIED => {
                    rdm_log::StatusLogger::modified_file(entry.path().unwrap())
                }
                Status::WT_DELETED => {
                    rdm_log::StatusLogger::removed_unsaved_file(
                        entry.path().unwrap(),
                    )
                }
                Status::INDEX_DELETED => {
                    rdm_log::StatusLogger::removed_file(entry.path().unwrap())
                }
                Status::IGNORED => (),
                _ => {
                    println!(
                        "File {} is something else",
                        entry.path().unwrap()
                    );
                }
            }
        }
    } else {
        println!("No changes since last save.")
    }

    Ok(())
}
