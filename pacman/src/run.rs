use crate::config;

pub fn run(path: &str, profile: &str, args: Vec<String>) {
    let config = config::read(path);
    let profile = match config.profile.get(profile) {
        Some(profile) => profile,
        None => {
            println!("Profile \"{}\" not found", profile);
            std::process::exit(1);
        }
    };
    println!("config: {:#?}", config);
}