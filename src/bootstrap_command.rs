use rdm_macros::{FromError, ToDoc};

use crate::config;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Bootstrap Error:"]
pub(crate) enum BootstrapError {
    #[doc_text = "The `bootstrap.lua' file was not found."]
    NoBootstrapFile,
    #[doc_to_string]
    LuaError(mlua::Error),
    #[doc_to_string]
    IoError(std::io::Error),
}

pub(crate) fn run(config: config::Config) -> Result<(), BootstrapError> {
    let mut lua_path = config.config_path.clone();
    lua_path.push("bootstrap.lua");

    if !lua_path.exists() {
        return Err(BootstrapError::NoBootstrapFile);
    }

    let lua = config.lua;

    let str = std::fs::read_to_string(&lua_path)?;

    lua.load(&str)
        .set_name(lua_path.to_str().unwrap())?
        .exec()?;
    Ok(())
}
