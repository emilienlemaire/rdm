use curl::easy::Easy;
use mlua::Lua;
use rdm_macros::{FromError, ToDoc};
use run_script::ScriptOptions;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Lua Runtime Error:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum RuntimeError {
    #[doc_to_string]
    LuaError(mlua::Error),
    #[doc_to_string]
    CurlError(curl::Error),
}

fn add_curl_function(lua: &Lua) -> Result<(), RuntimeError> {
    let curl_fn = lua.create_function(|_, url: String| {
        let mut easy = Easy::new();
        let mut buf = Vec::new();

        easy.fail_on_error(true).unwrap();
        easy.follow_location(true).unwrap();

        easy.url(url.as_str()).unwrap();

        {
            let mut transfer = easy.transfer();

            transfer
                .write_function(|data| {
                    buf.extend_from_slice(data);
                    Ok(data.len())
                })
                .unwrap();

            transfer.perform().unwrap();
        }

        let str = String::from_utf8(buf).unwrap();

        Ok(str)
    })?;

    let globals = lua.globals();

    globals.set("curl", curl_fn)?;

    Ok(())
}

fn add_run_function(lua: &Lua) -> Result<(), RuntimeError> {
    let run_fn = lua.create_function(|_, script: String| {
        let mut options = ScriptOptions::new();
        options.runner = Some("/bin/bash".to_string());
        options.output_redirection = run_script::IoOptions::Inherit;

        let args = vec![];

        let (code, output, error) =
            run_script::run(script.as_str(), &args, &options).unwrap();

        Ok((code, output, error))
    })?;

    lua.globals().set("run", run_fn)?;

    Ok(())
}

pub(crate) fn init_lua_runtime() -> Result<Lua, RuntimeError> {
    let lua = Lua::new();

    add_curl_function(&lua)?;
    add_run_function(&lua)?;

    Ok(lua)
}
