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
    // determine if we have to compile for current profile
    let mut compile = false;
    // check if there is directory for the profile
    let profile_path = std::path::Path::new(path).join("target").join(profile.0);
    if !profile_path.exists() {
        compile = true;
        // create directory
        std::fs::create_dir_all(&profile_path).unwrap();
    }
    

    println!("config: {:#?}", config);
}