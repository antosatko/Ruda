use std::collections::HashMap;

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub kind: ProjectKind,
    pub author: String,
    pub license: String,
    pub description: String,
    pub runtime: String,
    #[serde(rename = "3rdparty")]
    pub _3rdparty: _3rdparty,

    #[serde(default = "HashMap::new")]
    pub dependencies: HashMap<String, String>,

    #[serde(default = "Vec::new")]
    pub binaries: Vec<String>,

    #[serde(default = "HashMap::new")]
    pub profile: HashMap<String, Profile>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Profile {
    pub runtime: Option<String>,
    pub _3rdparty: Option<String>,
    #[serde(default = "HashMap::new")]
    pub dependencies: HashMap<String, String>,
    #[serde(default = "Vec::new")]
    pub binaries: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub enum ProjectKind {
    #[serde(rename = "lib")]
    Lib,
    #[serde(rename = "bin")]
    Bin,
}

#[derive(serde::Deserialize, Debug)]
pub enum _3rdparty {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "std")]
    Std,
    #[serde(rename = "sandboxed")]
    Sandboxed,
    #[serde(rename = "deny")]
    Deny,
}

/// Read a config file
/// panics:
///    - if the file does not exist
///   - if the file is not valid toml
/// 
pub fn read(path: &str) -> Config {
    let _path = std::path::Path::new(path).join("Ruda.toml");
    let config = match std::fs::read_to_string(_path) {
        Ok(config) => config,
        Err(_) => {
            println!("Failed to read config file at {}", path);
            std::process::exit(1);
        }
    };
    let config: Config = match toml::from_str(&config) {
        Ok(config) => config,
        Err(err) => {
            println!("{}", err);
            println!("Failed to parse config file at {}", path);
            std::process::exit(1);
        }
    };
    config
}