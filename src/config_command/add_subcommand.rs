use std::path::{Path, PathBuf};

use rdm_macros::{FromError, ToDoc};

use crate::config;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while runnit the `add' subcommand:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum AddSubcommandError {
    #[doc_to_string]
    GitError(git2::Error),
    #[doc_to_string]
    IoError(std::io::Error),
}

fn add_path(
    repo: &git2::Repository,
    index: &mut git2::Index,
    path: &Path,
) -> Result<(), AddSubcommandError> {
    let status = repo.status_file(path)?;

    if status.contains(git2::Status::WT_NEW)
        || status.contains(git2::Status::WT_MODIFIED)
    {
        index.add_path(path.as_ref())?;
        index.write()?;
        log::info!("Added file: {}", path.to_str().unwrap());
    } else {
        log::info!("File already added: {}", path.to_str().unwrap());
    }

    Ok(())
}

pub(super) fn run(
    config: config::Config,
    path: Vec<PathBuf>,
) -> Result<(), AddSubcommandError> {
    let repo = config.repo;

    let mut index = repo.index()?;

    println!("{:#?}", path);

    path.iter().try_for_each(|path| {
        let abs_path = std::fs::canonicalize(path)?;
        let rel_path =
            pathdiff::diff_paths(abs_path, &config.worktree_path).unwrap();
        add_path(&repo, &mut index, rel_path.as_path())
    })?;

    Ok(())
}
