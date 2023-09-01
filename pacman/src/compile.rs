use crate::{config, sum};

pub fn compile(path: &str, profile: (&str, &config::Profile)) {
    // determine if we have to compile for current profile
    let mut compile = false;
    // check if there is directory for the profile
    let profile_path = std::path::Path::new(path).join("target").join(profile.0);
    if !profile_path.exists() {
        compile = true;
        // create directory
        std::fs::create_dir_all(&profile_path).unwrap();
    }
    // check if there is a sums.txt file
    let sums_path = profile_path.join(sum::TARGET_FILE);
    if !sums_path.exists() {
        compile = true;
        sum::write_sums(path, profile.0, &sum::sum(path, profile.0));
    } else if !sum::check(path, profile.0) {
        compile = true;
        sum::write_sums(path, profile.0, &sum::sum(path, profile.0));
    }
    if !compile {
        return;
    }
    // compile
    println!("Compiling... {} {}", path, profile.0);
}
