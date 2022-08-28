use std::path::{Path, PathBuf};

use rdm_macros::{FromError, ToDoc};

use crate::config;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while running the `update' subcommand:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum UpdateSubcommandError {
    #[doc_to_string]
    GitError(git2::Error),
    #[doc_to_string]
    IoError(std::io::Error),
}

fn update_path(
    repo: &git2::Repository,
    index: &mut git2::Index,
    path: &Path,
) -> Result<(), UpdateSubcommandError> {
    let status = repo.status_file(path)?;

    if status.contains(git2::Status::WT_MODIFIED) {
        index.add_path(path.as_ref())?;
        index.write()?;

        log::info!(
            "The file or directory {} was updated to your configuration repository",
            path.to_str().unwrap()
        );
    }

    Ok(())
}

pub(super) fn run(
    config: config::Config,
    paths: Vec<PathBuf>,
) -> Result<(), UpdateSubcommandError> {
    let repo = config.repo;
    let mut index = repo.index()?;

    if paths.is_empty() {
        let mut opts = git2::StatusOptions::new();
        let status = repo.statuses(Some(&mut opts))?;
        for entry in status
            .iter()
            .filter(|e| e.status() != git2::Status::CURRENT)
        {
            let path = Path::new(entry.path().unwrap());
            update_path(&repo, &mut index, path)?;
        }
    } else {
        paths.iter().try_for_each(|path| {
            let abs_path = std::fs::canonicalize(path)?;
            let rel_path =
                pathdiff::diff_paths(abs_path, &config.worktree_path).unwrap();
            update_path(&repo, &mut index, rel_path.as_path())
        })?;
    }

    Ok(())
}
