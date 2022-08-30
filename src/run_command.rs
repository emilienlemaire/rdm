use rdm_macros::{FromError, ToDoc};

use crate::config::Config;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while running the run command:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum RunCommandError {
    #[doc_text = "The file `init.lua' was not found.`"]
    InitLuaNotFound,
    #[doc_to_string]
    IoError(std::io::Error),
    #[doc_to_string]
    LuaError(mlua::Error),
}

pub(crate) fn run(config: Config) -> Result<(), RunCommandError> {
    let mut lua_init_file = config.config_path;

    lua_init_file.push("init.lua");

    if !lua_init_file.exists() {
        return Err(RunCommandError::InitLuaNotFound);
    }

    let str = std::fs::read_to_string(lua_init_file)?;

    config.lua.load(str.as_str()).exec()?;

    Ok(())
}
