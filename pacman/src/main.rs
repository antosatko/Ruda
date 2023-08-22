use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[clap(
    name = "Ruda pacman",
    version = "0.1.0",
)]
struct Args {
    /// Task to perform
    #[command(subcommand)]
    task: Option<Task>,
}

#[derive(Debug, Subcommand)]
enum Task {
    /// Initialize a project
    Init {
        /// Project path
        #[clap(name = "path")]
        path: Option<String>,

        /// Project name
        #[clap(name = "name", short, long)]
        name: Option<String>,

        /// Project version
        #[clap(name = "version", short, long)]
        version: Option<String>,
    },
    /// Build a project from source and run it
    Run {
        /// Project path
        #[clap(name = "path")]
        path: Option<String>,

        /// Runtime arguments for the VM
        #[clap(name = "args", last = true)]
        args: Vec<String>,
    },
    /// Build a project from source
    Build {
        /// Project path
        #[clap(name = "path")]
        path: Option<String>,
    },
    /// Install a package
    Install {
        /// URL
        #[clap(name = "url")]
        url: String,

        /// version 
        #[clap(name = "version", short, long, default_value = "latest")]
        version: String,
    },
    /// Remove a package
    Remove {
        /// URL
        #[clap(name = "url")]
        url: String,

        /// version (default: all)
        #[clap(name = "version", short, long, default_value = "all")]
        version: String,
    },
    /// Update Ruda 
    Update {
        /// version 
        #[clap(name = "version", default_value = "latest")]
        ver: String,

        /// Check for updates only
        #[clap(short, long)]
        check: bool,
    },
}

fn main() {
    let args = Args::parse();

    println!("{:?}", args);
}