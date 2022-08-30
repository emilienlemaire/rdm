use curl::easy::Easy;
use mlua::Lua;
use rdm_macros::{FromError, ToDoc};

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Error while initializing the `curl' function: "]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub enum CurlFunctionError {
    #[doc_to_string]
    LuaError(mlua::Error),
    #[doc_to_string]
    CurlError(curl::Error),
}

pub fn curl_fn(lua: &Lua) -> Result<mlua::Function, CurlFunctionError> {
    let fun = lua.create_function(|_, url: String| {
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

    Ok(fun)
}
