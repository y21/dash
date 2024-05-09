use std::collections::HashMap;
use std::path::PathBuf;

use dash_proc_macro::Trace;
use serde::Deserialize;

fn default_main() -> PathBuf {
    "index.js".into()
}

#[derive(Deserialize, Debug, Trace)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default = "default_main")]
    pub main: PathBuf,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}
impl Package {
    pub fn default_with_entry(entry: PathBuf) -> Self {
        Package {
            name: String::default(),
            version: String::default(),
            description: String::default(),
            main: entry,
            dependencies: HashMap::default(),
        }
    }
}
