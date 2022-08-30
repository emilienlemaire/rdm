#![allow(clippy::enum_variant_names)]
mod functions;

use std::path::PathBuf;

use mlua::Lua;
use rdm_macros::{FromError, ToDoc};

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Lua Runtime Error:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub enum RuntimeError {
    RunScriptFunctionError(functions::run_script::RunScriptFunctionError),
    CurlFunctionError(functions::curl::CurlFunctionError),
    FileFunctionError(functions::file::FileFunctionError),
    DirectoryFunctionError(functions::directory::DirectoryFunctionError),
    #[doc_to_string]
    LuaError(mlua::Error),
}

pub fn init(
    repo_path: PathBuf,
    worktree_path: PathBuf,
) -> Result<Lua, RuntimeError> {
    let lua = Lua::new();

    lua.globals()
        .set("run_script", functions::run_script_fn(&lua)?)?;
    lua.globals().set("curl", functions::curl_fn(&lua)?)?;
    lua.globals().set(
        "file",
        functions::file_fn(&lua, repo_path.clone(), worktree_path.clone())?,
    )?;
    lua.globals().set(
        "directory",
        functions::directory_fn(&lua, repo_path, worktree_path)?,
    )?;

    Ok(lua)
}
