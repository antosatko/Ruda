mod args;
mod build;
mod compile;
mod config;
mod init;
mod remote;
mod sum;

use args::Task;
use clap::Parser;

fn main() {
    let args = args::Args::parse();
    match &args.task {
        Task::Init {
            kind,
            name,
            version,
            author,
            path,
        } => {
            init::init(
                &path,
                kind.clone(),
                name.clone(),
                version.clone(),
                author.clone(),
            );
        }
        Task::Run { profile, args, path } => {
            build::run(&path, profile, args.clone());
        }
        Task::Build { profile, path } => {
            build::build(&path, profile);
        }
        Task::Install { source, version } => {
            remote::install(source, version);
        }
        Task::Remove { source, version } => {
            remote::uninstall(source, version);
        }
        Task::Locate { url, version } => match url {
            Some(url) => println!("locate {}", remote::path(url, version)),
            None => println!("locate {}", std::env::var("RUDA_PATH").unwrap()),
        },
    }
}
