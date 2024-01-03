mod args;
mod build;
mod compile;
mod config;
mod init;
mod remote;
mod sum;
mod run;
mod lens;

use args::Task;
use clap::Parser;
use config::Profile;

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
        Task::Run {
            profile,
            args,
            path,
            debug,
        } => {
            build::run(&path, profile, args.clone(), *debug);
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
            None => println!("{}", std::env::var("RUDA_PATH").unwrap()),
        },
        Task::Restore {
            profile,
            path,
            compile,
            run,
            args,
            debug,
        } => {
            build::restore(&path, profile, *compile, *run, args.clone(), *debug);
        }
        Task::Lens { path, target, profile } => {
            let target = match target {
                Some(target) => target.clone(),
                None => args::LensTarget::Project,
            };
            match target {
                args::LensTarget::Bin => lens::bin(&path, (&profile, config::read(&path).profile.get(profile).expect("profile not found"))),
                args::LensTarget::Project => lens::project(&path, (&profile, config::read(&path).profile.get(profile).expect("profile not found"))),
                args::LensTarget::Std => lens::std(),
            }
        }
    }
}
