use crate::{config::Profile, build::path_to_exe};

pub fn run(path: &str, profile: &(&str, &Profile), _args: &Vec<String>){
    let path = path_to_exe(path, &profile);
    let mut args = _args.clone();
    args.insert(0, path.to_string());
    println!("Running: rudavm {:?}", args);
    let cmd = std::process::Command::new("rudavm").args(args).spawn();
    match cmd {
        Ok(mut cmd) => {
            cmd.wait().unwrap();
        }
        Err(err) => {
            println!("Failed to run program. {}", err);
        }
    }
}