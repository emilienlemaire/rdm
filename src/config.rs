use rdm_macros::{FromError, ToDoc};

use crate::args::Args;
use crate::lockfile::{self, TomlConfig};
use crate::{lua_runtime, utils};
use std::path::PathBuf;

pub(crate) struct Config {
    pub(crate) config_path: PathBuf,
    pub(crate) repo_path: PathBuf,
    pub(crate) worktree_path: PathBuf,
    pub(crate) repo: git2::Repository,
    pub(crate) lua: mlua::Lua,
    pub(crate) revision: u32,
}

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while running the config command:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum ConfigError {
    #[doc_text = "The `rdm.lock' file was not found."]
    NoLockFile,
    LockFileError(lockfile::LockFileError),
    #[doc_to_string]
    GitError(git2::Error),
    #[doc_to_string]
    IOError(std::io::Error),
    LuaRuntimeError(lua_runtime::RuntimeError),
}

impl Config {
    pub(crate) fn new(args: &Args) -> Result<Config, ConfigError> {
        let mut expanded: PathBuf = match &args.config_path {
            Some(config_path) => utils::full_expand(config_path.as_str()),
            None => utils::full_expand("~/.config/rdm/"),
        }
        .into();

        let config_path = expanded.clone();

        expanded.push("rdm.lock");

        if !expanded.exists() {
            log::error!("Lockfile not found: {}", expanded.to_str().unwrap());
            return Err(ConfigError::NoLockFile);
        }

        let TomlConfig {
            revision,
            repo_path,
            worktree_path,
        } = TomlConfig::load(&expanded)?;

        let repo_path: PathBuf = repo_path.into();
        let worktree_path: PathBuf = worktree_path.into();
        let repo = git2::Repository::open_bare(&repo_path)?;
        repo.set_workdir(worktree_path.as_path(), false)?;

        let lua = lua_runtime::init_lua_runtime()?;

        Ok(Config {
            config_path,
            repo_path,
            repo,
            revision,
            worktree_path,
            lua,
        })
    }
}
