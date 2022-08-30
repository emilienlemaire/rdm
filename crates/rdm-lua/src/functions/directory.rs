use std::{
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

use git2::{Repository, Status};
use mlua::{Error, Function, Lua};
use rdm_macros::{FromError, ToDoc};

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while initializing the `directory' function: "]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub enum DirectoryFunctionError {
    #[doc_to_string]
    LuaError(mlua::Error),
}

pub fn directory_fn(
    lua: &Lua,
    repo_path: PathBuf,
    worktree_path: PathBuf,
) -> Result<Function, DirectoryFunctionError> {
    fn get_files(dir: &PathBuf) -> Vec<PathBuf> {
        let mut files = Vec::new();

        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                files.append(&mut get_files(&path));
            } else {
                files.push(path);
            }
        }

        files
    }

    // TODO: Add an ignore parameter.
    let directory_fn = lua.create_function(move |_, directory: String| {
        let path = Path::new(&directory);
        let abs_dir = std::fs::canonicalize(path).unwrap();
        let abs_wt = std::fs::canonicalize(&worktree_path).unwrap();

        if !&abs_dir.exists() {
            return Err(Error::external(format!(
                "File or directory not found: {}.",
                directory
            )));
        }

        if !&abs_dir.is_dir() {
            return Err(Error::external(format!(
                "The path {} does not point to a directory",
                directory
            )));
        }

        let files = get_files(&abs_dir);

        let repo = Repository::open_bare(&repo_path).unwrap();
        repo.set_workdir(&worktree_path, false).unwrap();

        let mut index = repo.index().unwrap();

        files.iter().for_each(|file| {
            let path = pathdiff::diff_paths(file, &abs_wt).unwrap();
            let status = repo.status_file(&path).unwrap();
            match status {
                Status::WT_NEW
                | Status::WT_DELETED
                | Status::WT_RENAMED
                | Status::WT_MODIFIED
                | Status::WT_TYPECHANGE => {
                    index.add_path(&path).unwrap();
                    index.write().unwrap();
                    log::info!(
                        "The file {} was added to your config.",
                        path.to_str().unwrap()
                    );
                }
                _ => (),
            };
        });

        Ok(())
    })?;

    Ok(directory_fn)
}
