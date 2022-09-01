use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Path to the configuration to work with, default is `$HOME/.config/rdm`
    #[clap(short, long, value_parser)]
    pub config_path: Option<String>,
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Initialize the rdm folders and local repository.
    Init {
        /// Path to the local bare repository to create, if none is provided then then default is
        /// `~/config/rdm/repo`.
        #[clap(long, short, value_parser)]
        repo: Option<String>,
        /// If given, then a default configuration file will be created at this location, if not
        /// then if there are no files at `~/.config/rdm/init.lua` the default file will be created
        /// at this location.
        #[clap(long, short, value_parser)]
        config: Option<String>,
        /// Path of the git worktree, if none is given then it defaults to your
        /// current directory.
        #[clap(long, short, value_parser)]
        worktree: Option<String>,
    },
    /// Manage your configuration
    #[clap(subcommand)]
    Config(ConfigSubCommand),
    /// Run the `bootstrap.lua` file.
    Bootstrap,
    /// Run the `init.lua` file.
    Run,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigSubCommand {
    /// Add files or folders to your config
    Add {
        /// Path for files or folder ot be added to the configuration
        #[clap(required = true, value_parser)]
        path: Vec<PathBuf>,
    },
    /// Stage the changes
    Update {
        /// If given only stage the paths given, otherwise stage all the changes
        #[clap(value_parser)]
        path: Vec<PathBuf>,
    },
    /// Show the status of your current config, the `(unsaved)' flag note the
    /// files that have not been updated yet in your config, run `config update`
    /// to fix it.
    Status {
        /// If you want to show untracked files of your config
        #[clap(value_parser, short, long)]
        untracked: bool,
    },
    /// Save the current state of your config. This will create a commit in the
    /// your local repository with a generated message.
    Save,
    /// Manage the config remotes
    #[clap(subcommand)]
    Remote(RemoteSubCommand),
}

#[derive(Debug, Subcommand)]
pub(crate) enum RemoteSubCommand {
    /// Add a remote to your config.
    Add {
        /// The name of the remote
        name: String,
        /// The url of the remote
        url: String,
        // TODO: Figure this out
        // #[clap(short, long, value_parser)]
        // default: bool,
    },
    /// Remove a remote from your config.
    Remove { name: String },
    // TODO: Figure this out
    // Default { name: String },
    /// List all remotes
    List,
}
