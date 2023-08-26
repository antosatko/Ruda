mod init;
mod args;
mod sum;
mod config;
mod run;

use args::Task;
use clap::Parser;

fn main() {
    let args = args::Args::parse();
    match &args.task {
        Task::Init { kind, name, version, author } => {
            init::init(".", kind.clone(), name.clone(), version.clone(), author.clone());
        }
        Task::Run { profile, args } => {
            run::run(".", profile, args.clone());
        }
        _ => println!("{:?}", args),
    }
}