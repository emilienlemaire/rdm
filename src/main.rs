#![allow(clippy::enum_variant_names)]
mod args;
mod bootstrap_command;
mod config;
mod config_command;
mod init_command;
mod lockfile;
mod rdm_error;
mod run_command;
mod utils;

use std::process::exit;

use clap::Parser;

use args::{Args, Commands};
use config::Config;
use rdm_error::RdmError;

//IDEAS:
//  * Manage hosts by branches
//  * Enable copy paste in Lua
fn main() {
    rdm_log::MainLogger::set_as_logger().unwrap();

    let args = Args::parse();

    let res = if let Commands::Init {
        repo,
        config,
        worktree,
    } = args.command
    {
        init_command::run(repo, config, worktree).map_err(RdmError::from)
    } else {
        match Config::new(&args) {
            Err(err) => Err(err.into()),
            Ok(config) => match args.command {
                Commands::Config(sub_command) => {
                    config_command::run(sub_command, config)
                        .map_err(RdmError::from)
                }
                Commands::Bootstrap => {
                    bootstrap_command::run(config).map_err(RdmError::from)
                }
                _ => Ok(()),
            },
        }
    };

    match res {
        Err(err) => {
            let (_, cols) = console::Term::stdout().size();
            log::error!("{}", err.to_pretty(cols.into()));
            exit(1)
        }
        Ok(()) => exit(0),
    }
}
