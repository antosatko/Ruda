mod init;
mod args;

use args::Task;
use clap::Parser;

fn main() {
    let args = args::Args::parse();
    match &args.task {
        Task::Init { kind, name, version, author } => {
            init::init(".", kind.clone(), name.clone(), version.clone(), author.clone());
        }
        _ => println!("{:?}", args),
    }
}