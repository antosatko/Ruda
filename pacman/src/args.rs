use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[clap(name = "Ruda pacman", version = "0.1.0")]
pub struct Args {
    /// Task to perform
    #[command(subcommand)]
    pub task: Task,
}

#[derive(Debug, Subcommand)]
pub enum Task {
    /// Initialize a project
    Init {
        /// Kind of project
        #[clap(name = "kind", default_value = "bin", short, long)]
        kind: ProjectKind,

        /// Project name
        #[clap(name = "name", short, long)]
        name: Option<String>,

        /// Project version
        #[clap(name = "version", short, long)]
        version: Option<String>,

        /// Project author
        #[clap(name = "author", short, long)]
        author: Option<String>,
    },
    /// Build a project from source and run it
    Run {
        /// Profile to use
        #[clap(name = "profile", short, long, default_value = "default")]
        profile: String,

        /// Runtime arguments for the VM
        #[clap(name = "args", last = true)]
        args: Vec<String>,
    },
    /// Build a project from source
    Build {
        /// Profile to use
        #[clap(name = "profile", short, long, default_value = "default")]
        profile: String,
    },
    /// Install a package
    Install {
        /// source URL or path
        #[clap(name = "source")]
        source: String,

        /// version
        #[clap(name = "version", short, long, default_value = "latest")]
        version: String,
    },
    /// Remove a package
    Remove {
        /// source URL or path
        #[clap(name = "source")]
        source: String,

        /// version
        #[clap(name = "version", short, long, default_value = "all")]
        version: String,
    },
    /// Locates a package using url
    Locate {
        /// URL
        /// if no url is provided, outputs the path to the ruda root directory
        #[clap(name = "url")]
        url: Option<String>,

        /// version
        #[clap(name = "version", short, long, default_value = "latest")]
        version: String,
    },
}

#[derive(Debug, ValueEnum, Clone)]
pub enum ProjectKind {
    /// A library
    Lib,
    /// An executable
    Bin,
}
