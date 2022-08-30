use std::path::{Path, PathBuf};

use git2::{Repository, Status};
use mlua::{Function, Lua};
use rdm_macros::{FromError, ToDoc};

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while initializing the `file' function: "]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub enum FileFunctionError {
    #[doc_to_string]
    LuaError(mlua::Error),
}

pub fn file_fn(
    lua: &Lua,
    repo_path: PathBuf,
    worktree_path: PathBuf,
) -> Result<Function, FileFunctionError> {
    let file_fn = lua.create_function(move |_, file: String| {
        let path = Path::new(&file);
        let repo = Repository::open_bare(&repo_path).unwrap();
        repo.set_workdir(&worktree_path, false).unwrap();

        let status = repo.status_file(path).unwrap();
        let mut index = repo.index().unwrap();

        match status {
            Status::WT_NEW
            | Status::WT_DELETED
            | Status::WT_RENAMED
            | Status::WT_MODIFIED
            | Status::WT_TYPECHANGE => {
                log::info!("The file {} was added to your config.", file);
                index.add_path(path).unwrap();
                index.write().unwrap();
            }
            _ => (),
        };

        Ok(())
    })?;

    Ok(file_fn)
}
