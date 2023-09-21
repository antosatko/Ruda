use crate::compile;
use crate::config;
use crate::config::Profile;
use crate::remote;

pub fn run(path: &str, profile: &str, _args: Vec<String>) {
    let config = config::read(path);
    let profile = match config.profile.get(profile) {
        Some(prof) => (profile, prof),
        None => {
            println!("Profile \"{}\" not found", profile);
            std::process::exit(1);
        }
    };
    // build dependencies
    build_deps(&profile.1, profile.1._3rdparty as usize);
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
    build_deps(&profile.1, config._3rdparty as usize);
    // compile
    compile::compile(path, profile);

    println!("config: {:#?}", config);
}

/// Build dependencies for a profile
pub fn build_deps(profile: &Profile, _3rdparty: usize) {
    for dep in &profile.dependencies {
        let mut path = remote::path(&dep.1.path, &"latest");
        // exists?
        if !std::path::Path::new(&path).exists() {
            if remote::is_remote(&dep.1.path) {
                path = remote::install(&dep.1.path, &"latest");
            } else {
                println!("Dependency {} not found", dep.1.path);
                std::process::exit(1);
            }
        }
        // is it a package?
        if config::contains(&path) {
            // read config
            let config = config::read(&path);
            // check 3rdparty level
            // levels: 0 - allow, 1 - std, 2 - sandboxed, 3 - deny
            let this_3rdparty = config._3rdparty as usize;
            // only for debug
            if this_3rdparty < _3rdparty {
                println!("Dependency {} is not allowed", dep.1.path);
                println!(
                    "{} is \"{}\" level, but current project allows \"{}\" level",
                    dep.1.path,
                    config::_3rdparty::to_str(this_3rdparty),
                    config::_3rdparty::to_str(_3rdparty)
                );
                std::process::exit(1);
            }
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
            build_deps(&profile.1, this_3rdparty);
            // compile
            compile::compile(&path, (profile.0, profile.1));
        } else {
            // err
            println!("Dependency {} is not a package", dep.1.path);
            std::process::exit(1);
        }
    }
}
