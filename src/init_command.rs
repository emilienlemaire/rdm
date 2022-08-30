use std::{fs::File, io::Write, path::PathBuf};

use git2::{Repository, RepositoryInitOptions};
use rdm_macros::{FromError, ToDoc};

use crate::lockfile;
use crate::lockfile::TomlConfig;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while running the `init' command:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum InitError {
    #[doc_to_string]
    IoError(std::io::Error),
    #[doc_to_string]
    GitError(git2::Error),
    #[doc_format(
        format_str = "The path `{}' does not point to a directory.",
        _1
    )]
    #[no_from]
    IsNotDirError(String),
    #[doc_format(
        format_str = "The path `{}' does not point to a lua file.",
        _1
    )]
    #[no_from]
    ConfigFileNotLuaError(String),
    LockFileError(lockfile::LockFileError),
}

pub(crate) fn run(
    repo_path: Option<String>,
    config_path: Option<String>,
    worktree: Option<String>,
) -> Result<(), InitError> {
    let repo_path_buf = match &repo_path {
        None => {
            let current_dir = std::env::current_dir();
            current_dir?
        }
        Some(path) => PathBuf::from(path),
    };

    if !repo_path_buf.exists() {
        log::info!("Creating directory {}", repo_path_buf.display());
        std::fs::create_dir_all(&repo_path_buf)?;
    }

    let repo_path_buf = std::fs::canonicalize(repo_path_buf)?;

    if !repo_path_buf.is_dir() {
        return Err(InitError::IsNotDirError(
            repo_path_buf.to_str().unwrap().to_string(),
        ));
    }

    let config_path_buf = match &config_path {
        Some(path) => PathBuf::from(path),
        None => {
            let home = std::env!("HOME");
            let mut path = PathBuf::from(home);
            path.push(".config/rdm/rdm.lua");
            path
        }
    };

    match config_path_buf.extension() {
        Some(ext) => {
            if ext != "lua" {
                return Err(InitError::ConfigFileNotLuaError(
                    config_path_buf.into_os_string().into_string().unwrap(),
                ));
            }
        }
        _ => {
            return Err(InitError::ConfigFileNotLuaError(
                config_path_buf.into_os_string().into_string().unwrap(),
            ))
        }
    };

    let worktree_path = match &worktree {
        Some(str) => {
            let path = PathBuf::from(str);
            if !path.exists() {
                log::info!("Creating worktree path {}", path.display());
                std::fs::create_dir_all(&path)?;
            }
            std::fs::canonicalize(path)?
        }
        None => std::env::current_dir()?,
    };

    let mut opts = RepositoryInitOptions::new();

    opts.bare(true);

    log::info!("Creating bare repository at {}", repo_path_buf.display());
    opts.workdir_path(worktree_path.as_path());
    let repo = Repository::init_opts(&repo_path_buf, &opts)?;
    log::info!("Bare repository created at {}", repo_path_buf.display());

    if !config_path_buf.exists() {
        match config_path_buf.parent() {
            None => (),
            Some(parent) => {
                log::info!("Creating directory {}", parent.display());
                std::fs::create_dir_all(parent)?
            }
        };
        log::info!("Creating file {}", config_path_buf.display());
        let _ = File::create(&config_path_buf)?;
    }

    let mut gitgnore_path = worktree_path.clone();

    gitgnore_path.push(".gitignore");

    let mut gitignore = File::create(&gitgnore_path)?;

    log::info!("Adding repo directory to {}", gitgnore_path.display());

    gitgnore_path.pop();

    let rel_repo_path =
        pathdiff::diff_paths(repo_path_buf, &gitgnore_path).unwrap();
    writeln!(&mut gitignore, "{}", rel_repo_path.display())?;

    repo.config()?
        .open_level(git2::ConfigLevel::Local)?
        .set_str("status.showUntrackedFiles", "no")?;
    repo.set_workdir(worktree_path.as_path(), false)?;

    let mut index = repo.index()?;
    gitgnore_path.push(".gitignore");
    let rel_gitignore_path =
        pathdiff::diff_paths(gitgnore_path, &worktree_path).unwrap();

    let lock_path = match &config_path {
        Some(path) => {
            let mut path = PathBuf::from(path);
            path.pop();
            path.push("rdm.lock");
            path
        }
        None => {
            let home = std::env!("HOME");
            let mut path = PathBuf::from(home);
            path.push(".config/rdm/rdm.lock");
            std::fs::canonicalize(path)?
        }
    };

    let conf = TomlConfig::new(&repo_path, &worktree_path.to_str())?;
    conf.save(&lock_path)?;

    let abs_worktree = std::fs::canonicalize(&worktree_path)?;
    let abs_lock_path = std::fs::canonicalize(&lock_path)?;
    let abs_config_path = std::fs::canonicalize(&config_path_buf)?;

    index
        .add_path(&pathdiff::diff_paths(abs_lock_path, &abs_worktree).unwrap())
        .unwrap();
    index
        .add_path(
            &pathdiff::diff_paths(abs_config_path, &abs_worktree).unwrap(),
        )
        .unwrap();
    index.add_path(rel_gitignore_path.as_path()).unwrap();
    index.write().unwrap();

    repo.commit(
        Some("HEAD"),
        &repo.signature()?,
        &repo.signature()?,
        "Initial commit",
        &repo.find_tree(index.write_tree()?)?,
        &[],
    )
    .unwrap();

    log::info!("Initial commit created.");

    Ok(())
}
