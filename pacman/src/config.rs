use serde_either::StringOrStruct;
use std::collections::HashMap;

use crate::remote;

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
    dependencies: HashMap<String, StringOrStruct<TempDependencyTable>>,

    #[serde(default = "HashMap::new")]
    binaries: HashMap<String, String>,

    #[serde(default = "HashMap::new")]
    profile: HashMap<String, TempProfile>,
}

#[derive(serde::Deserialize, Debug)]
struct TempProfile {
    runtime: Option<String>,
    _3rdparty: Option<_3rdparty>,
    #[serde(default = "HashMap::new")]
    dependencies: HashMap<String, StringOrStruct<TempDependencyTable>>,
    #[serde(default = "HashMap::new")]
    binaries: HashMap<String, String>,
}

/// describes a dependency
#[derive(serde::Deserialize, Debug, Clone)]
struct TempDependencyTable {
    version: Option<String>,
    path: Option<String>,
    profile: Option<String>,
    #[serde(rename = "3rdparty")]
    _3rdparty: Option<_3rdparty>,
    args: Option<Vec<String>>,
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

    pub dependencies: HashMap<String, Dependency>,
    pub binaries: HashMap<String, String>,

    pub profile: HashMap<String, Profile>,
}

#[derive(Debug)]
pub enum Runtime {
    Latest,
    Version(usize, usize, usize),
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub version: String,
    pub path: String,
    pub profile: String,
    pub _3rdparty: _3rdparty,
    pub args: Vec<String>,
}

#[allow(unused)]
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
    pub dependencies: HashMap<String, Dependency>,
    pub binaries: HashMap<String, String>,
}

fn temp_into_config(path: &str, temp: TempConfig) -> Config {
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
        binaries: HashMap::new(),
        profile: HashMap::new(),
    };

    config.dependencies = canonicalize_dependencies(path, &temp.dependencies);
    config.binaries = temp.binaries;
    config.profile = temp
        .profile
        .into_iter()
        .map(|(name, profile)| {
            (
                name,
                Profile {
                    runtime: profile.runtime.unwrap(),
                    _3rdparty: profile._3rdparty.unwrap(),
                    dependencies: canonicalize_dependencies(&path, &profile.dependencies),
                    binaries: profile.binaries,
                },
            )
        })
        .collect();

    config
}
fn fix_path(project: &str, path: &str) -> String {
    if remote::is_remote(path) {
        return path.to_string();
    }
    let path = std::path::Path::new(path);
    if path.is_relative() {
        let path = std::path::Path::new(project).join(path);
        path.to_str().unwrap().to_string()
    } else {
        path.to_str().unwrap().to_string()
    }
}

fn canonicalize_dependencies(
    path: &str,
    temp: &HashMap<String, StringOrStruct<TempDependencyTable>>,
) -> HashMap<String, Dependency> {
    let mut dependencies = HashMap::new();
    for (name, dependency) in temp.into_iter() {
        let dependency = match &dependency {
            // if path is relative then make it absolute by joining it with the current project path
            StringOrStruct::String(_path) => Dependency {
                version: String::from("latest"),
                path: fix_path(path, _path),
                profile: String::from("default"),
                _3rdparty: _3rdparty::Allow,
                args: Vec::new(),
            },
            StringOrStruct::Struct(dependency) => Dependency {
                version: dependency.version.clone().unwrap_or(String::from("latest")),
                path: fix_path(path, &dependency
                    .path
                    .clone()
                    .expect(format!("Dependency {} does not have a path", name).as_str())),
                profile: dependency
                    .profile
                    .clone()
                    .unwrap_or(String::from("default")),
                _3rdparty: dependency._3rdparty.clone().unwrap_or(_3rdparty::Allow),
                args: dependency.args.clone().unwrap_or(Vec::new()),
            },
        };
        dependencies.insert(name.to_string(), dependency);
    }
    dependencies
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
    let _path = std::path::Path::new(&std::env::var("RUDA_PATH").unwrap()).join("Ruda.toml");
    let global_config = match std::fs::read_to_string(&_path) {
        Ok(config) => config,
        Err(_) => {
            println!("Failed to read global config file at {}", _path.display());
            std::process::exit(1);
        }
    };
    let global_config: GlobalConfig = match toml::from_str(&global_config) {
        Ok(config) => config,
        Err(err) => {
            println!("{}", err);
            println!("Failed to parse global config file at {}", _path.display());
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
        // merge global config into profile dependencies
        // rule is: profile dependencies > global dependencies
        for (name, dependency) in config.dependencies.iter() {
            if profile.dependencies.get(name).is_none() {
                profile
                    .dependencies
                    .insert(name.clone(), dependency.clone());
            }
        }
        // merge global config into profile binaries
        // rule is: profile binaries > global binaries
        for (name, path) in config.binaries.iter() {
            if profile.binaries.get(name).is_none() {
                profile.binaries.insert(name.clone(), path.clone());
            }
        }
    }

    temp_into_config(&path, config)
}

pub fn contains(path: &str) -> bool {
    let path = std::path::Path::new(path).join("Ruda.toml");
    path.exists()
}
