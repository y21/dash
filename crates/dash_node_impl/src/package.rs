use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub main: String,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}
