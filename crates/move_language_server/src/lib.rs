use serde::de::DeserializeOwned;
use std::fmt;

mod global_state;
mod lsp_ext;
mod line_index;
mod handlers;
mod to_proto;
mod diagnostics;
mod mem_docs;
mod thread_pool;
mod caps;
mod lsp_utils;
mod from_proto;
mod dispatch;
mod reload;
mod main_loop;
pub mod config;

pub use main_loop::main_loop;
pub use caps::server_capabilities;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn from_json<T: DeserializeOwned>(what: &'static str, json: serde_json::Value) -> Result<T> {
    let res = serde_json::from_value(json.clone())
        .map_err(|e| format!("Failed to deserialize {}: {}; {}", what, e, json))?;
    Ok(res)
}

#[derive(Debug)]
struct LspError {
    code: i32,
    message: String,
}

impl LspError {
    fn new(code: i32, message: String) -> LspError {
        LspError { code, message }
    }
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Language Server request failed with {}. ({})", self.code, self.message)
    }
}

impl std::error::Error for LspError {}
