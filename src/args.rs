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
        /// your current directory.
        #[clap(long, short, value_parser)]
        repo: Option<String>,
        /// If given, then a default configuration file will be created at this location, if not
        /// then if there are no files at `~/.config/rdm/init.lua` the default file will be created
        /// at this location.
        #[clap(long, short, value_parser)]
        config: Option<String>,
        /// Path of the git worktree, if none is given then it defaults to `$HOME`.
        #[clap(long, short, value_parser)]
        worktree: Option<String>,
    },
    /// Manage your configuration
    #[clap(subcommand)]
    Config(ConfigSubCommand),
    /// Run the `bootstrap.lua` file.
    Bootstrap,
    /// Pull your configuration from a remote repository
    Pull {
        /// Select the name of the remote repository from your configuration file.
        #[clap(value_parser)]
        remote: Option<String>,
    },
    /// Push your configuration to a remote repository. If new changes have been made since the
    /// last synchronization, then they are committed.
    Push {
        /// Select the name of the remote repository from your configuration file.
        #[clap(value_parser)]
        remote: Option<String>,
        /// Ignore the synchronization
        #[clap(short, long, value_parser)]
        no_sync: bool,
        /// Override the default synchronization message, is ignored if --no-sync or no changes
        /// have been made since the last synchronization.
        #[clap(short, long, value_parser)]
        message: Option<String>,
    },
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
}
