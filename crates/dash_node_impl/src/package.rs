use std::collections::HashMap;

use dash_proc_macro::Trace;
use serde::Deserialize;

#[derive(Deserialize, Debug, Trace)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub main: String,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}
