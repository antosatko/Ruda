use crate::compile;
use crate::sum;
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
    // compile
    compile::compile(path, profile);

    println!("config: {:#?}", config);
}