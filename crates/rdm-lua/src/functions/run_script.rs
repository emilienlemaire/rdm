use mlua::Lua;
use rdm_macros::{FromError, ToDoc};
use run_script::ScriptOptions;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while initializing the `curl' function: "]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub enum RunScriptFunctionError {
    #[doc_to_string]
    LuaError(mlua::Error),
}

pub fn run_script_fn(
    lua: &Lua,
) -> Result<mlua::Function, RunScriptFunctionError> {
    let run_fn = lua.create_function(|_, script: String| {
        let mut options = ScriptOptions::new();
        options.runner = Some("/bin/bash".to_string());
        options.output_redirection = run_script::IoOptions::Inherit;

        let args = vec![];

        let (code, output, error) =
            run_script::run(script.as_str(), &args, &options).unwrap();

        Ok((code, output, error))
    })?;

    Ok(run_fn)
}
