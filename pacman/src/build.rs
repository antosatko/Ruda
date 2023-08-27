use crate::compile;
use crate::config::Profile;
use crate::remote;
use crate::config;

pub fn run(path: &str, profile: &str, args: Vec<String>) {
    let config = config::read(path);
    let profile = match config.profile.get(profile) {
        Some(prof) => (profile, prof),
        None => {
            println!("Profile \"{}\" not found", profile);
            std::process::exit(1);
        }
    };
    // build dependencies
    build_deps(&profile.1);
    // compile
    compile::compile(path, profile);

    // todo: run


    println!("config: {:#?}", config);
}

pub fn build(path: &str, profile: &str) {
    let config = config::read(path);
    let profile = match config.profile.get(profile) {
        Some(prof) => (profile, prof),
        None => {
            println!("Profile \"{}\" not found", profile);
            std::process::exit(1);
        }
    };
    // build dependencies
    build_deps(&profile.1);
    // compile
    compile::compile(path, profile);

    println!("config: {:#?}", config);
}


/// Build dependencies for a profile
pub fn build_deps(profile: &Profile) {
    for dep in &profile.dependencies {
        let mut path = remote::path(&dep.1.path, &"latest");
        // exists?
        if !std::path::Path::new(&path).exists() {
            if remote::is_remote(&dep.1.path) {
                path = remote::install(&dep.1.path, &"latest");
            }else {
                println!("Dependency {} not found", dep.1.path);
                std::process::exit(1);
            }
        }
        // is it a package?
        if config::contains(&path) {
            // read config
            let config = config::read(&path);
            // get profile
            // todo: get profile (default profile for now)
            let profile = match config.profile.get("default") {
                Some(prof) => (dep.0, prof),
                None => {
                    println!("Profile \"{}\" not found in {}", "default", path);
                    std::process::exit(1);
                }
            };
            // build dependencies
            build_deps(&profile.1);
            // compile
            compile::compile(&path, (profile.0, profile.1));
        }else {
            // err
            println!("Dependency {} is not a package", dep.1.path);
            std::process::exit(1);
        }
    }
}