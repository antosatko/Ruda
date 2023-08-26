use std::collections::HashMap;

#[derive(serde::Deserialize, Debug)]
struct TempConfig {
    name: Option<String>,
    version: Option<String>,
    kind: Option<ProjectKind>,
    author: Option<String>,
    license: Option<String>,
    description: Option<String>,
    runtime: Option<String>,
    #[serde(rename = "3rdparty")]
    _3rdparty: Option<_3rdparty>,

    #[serde(default = "HashMap::new")]
    dependencies: HashMap<String, String>,

    #[serde(default = "Vec::new")]
    binaries: Vec<String>,

    #[serde(default = "HashMap::new")]
    profile: HashMap<String, TempProfile>,
}

#[derive(serde::Deserialize, Debug)]
struct TempProfile {
    runtime: Option<String>,
    _3rdparty: Option<_3rdparty>,
    #[serde(default = "HashMap::new")]
    dependencies: HashMap<String, String>,
    #[serde(default = "Vec::new")]
    binaries: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub enum ProjectKind {
    #[serde(rename = "lib")]
    Lib,
    #[serde(rename = "bin")]
    Bin,
}

#[derive(serde::Deserialize, Debug, Clone)]
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

#[derive(serde::Deserialize, Debug)]
struct GlobalConfig {
    author: String,
    license: String,
    name: String,
    runtime: String,
    version: String,
    kind: ProjectKind,
    description: String,
    #[serde(rename = "3rdparty")]
    _3rdparty: _3rdparty,
}

#[derive(Debug)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub kind: ProjectKind,
    pub author: String,
    pub license: String,
    pub description: String,
    pub runtime: Runtime,
    pub _3rdparty: _3rdparty,

    pub dependencies: HashMap<String, String>,
    pub binaries: Vec<String>,

    pub profile: HashMap<String, Profile>,
}

#[derive(Debug)]
pub enum Runtime {
    Latest,
    Version(usize, usize, usize),
}

impl Runtime {
    pub fn from_str(s: &str) -> Runtime {
        if s == "latest" {
            return Runtime::Latest;
        }
        let mut s = s.split('.');
        let major = s.next().unwrap().parse::<usize>().unwrap();
        let minor = s.next().unwrap().parse::<usize>().unwrap();
        let patch = s.next().unwrap().parse::<usize>().unwrap();
        Runtime::Version(major, minor, patch)
    }
    pub fn to_string(&self) -> String {
        match self {
            Runtime::Latest => Runtime::latest().to_string(),
            Runtime::Version(major, minor, patch) => format!("{}.{}.{}", major, minor, patch),
        }
    }
    fn latest() -> Runtime {
        // TODO: get latest installed version
        Runtime::Version(0, 0, 0)
    }
}

#[derive(Debug)]
pub struct Profile {
    pub runtime: String,
    pub _3rdparty: _3rdparty,
    pub dependencies: HashMap<String, String>,
    pub binaries: Vec<String>,
}

fn temp_into_config(temp: TempConfig) -> Config {
    let mut config = Config {
        name: temp.name.unwrap(),
        version: temp.version.unwrap(),
        kind: temp.kind.unwrap(),
        author: temp.author.unwrap(),
        license: temp.license.unwrap(),
        description: temp.description.unwrap(),
        runtime: Runtime::from_str(&temp.runtime.unwrap()),
        _3rdparty: temp._3rdparty.unwrap(),
        dependencies: HashMap::new(),
        binaries: Vec::new(),
        profile: HashMap::new(),
    };
    
    config.dependencies = temp.dependencies;
    config.binaries = temp.binaries;
    config.profile = temp.profile.into_iter().map(|(name, profile)| (name, Profile {
        runtime: profile.runtime.unwrap(),
        _3rdparty: profile._3rdparty.unwrap(),
        dependencies: profile.dependencies,
        binaries: profile.binaries,
    })).collect();

    config
}

/// Read a config file
/// panics:
///    - if the file does not exist
///   - if the file is not valid toml
/// 
pub fn read(path: &str) -> Config {
    // read config file for the current project
    let _path = std::path::Path::new(path).join("Ruda.toml");
    let config = match std::fs::read_to_string(_path) {
        Ok(config) => config,
        Err(_) => {
            println!("Failed to read config file at {}", path);
            std::process::exit(1);
        }
    };
    let mut config: TempConfig = match toml::from_str(&config) {
        Ok(config) => config,
        Err(err) => {
            println!("{}", err);
            println!("Failed to parse config file at {}", path);
            std::process::exit(1);
        }
    };
    // read global config file
    let path = std::path::Path::new(&std::env::var("RUDA_PATH").unwrap()).join("Ruda.toml");
    let global_config = match std::fs::read_to_string(&path) {
        Ok(config) => config,
        Err(_) => {
            println!("Failed to read global config file at {}", path.display());
            std::process::exit(1);
        }
    };
    let global_config: GlobalConfig = match toml::from_str(&global_config) {
        Ok(config) => config,
        Err(err) => {
            println!("{}", err);
            println!("Failed to parse global config file at {}", path.display());
            std::process::exit(1);
        }
    };
    // merge global config into project config
    if config.author.is_none() {
        config.author = Some(global_config.author);
    }
    if config.license.is_none() {
        config.license = Some(global_config.license);
    }
    if config.name.is_none() {
        config.name = Some(global_config.name);
    }
    if config.runtime.is_none() {
        config.runtime = Some(global_config.runtime);
    }
    if config.version.is_none() {
        config.version = Some(global_config.version);
    }
    if config.kind.is_none() {
        config.kind = Some(global_config.kind);
    }
    if config.description.is_none() {
        config.description = Some(global_config.description);
    }
    if config._3rdparty.is_none() {
        config._3rdparty = Some(global_config._3rdparty);
    }
    // merge global config into profile config
    for (_, profile) in config.profile.iter_mut() {
        if profile.runtime.is_none() {
            profile.runtime = config.runtime.clone();
        }
        if profile._3rdparty.is_none() {
            profile._3rdparty = config._3rdparty.clone();
        }
        if profile.dependencies.is_empty() {
            profile.dependencies = config.dependencies.clone();
        }
        if profile.binaries.is_empty() {
            profile.binaries = config.binaries.clone();
        }
    }

    temp_into_config(config)
}