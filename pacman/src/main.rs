mod init;
mod args;
mod sum;
mod config;
mod build;
mod compile;
mod remote;

use args::Task;
use clap::Parser;

fn main() {
    let args = args::Args::parse();
    match &args.task {
        Task::Init { kind, name, version, author } => {
            init::init(".", kind.clone(), name.clone(), version.clone(), author.clone());
        }
        Task::Run { profile, args } => {
            build::run(".", profile, args.clone());
        }
        Task::Build { profile } => {
            build::build(".", profile);
        }
        Task::Install { source, version } => {
            remote::install(source, version);
        }
        Task::Remove { source, version } => {
            remote::uninstall(source, version);
        }
        Task::Locate { url, version } => {
            match url {
                Some(url) => println!("locate {}", remote::path(url, version)),
                None => println!("locate {}", std::env::var("RUDA_PATH").unwrap()),
            }
        }
        _ => println!("{:?}", args),
    }
}