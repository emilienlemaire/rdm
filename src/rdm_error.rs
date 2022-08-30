use rdm_macros::{FromError, ToDoc};

use crate::{
    bootstrap_command, config, config_command, init_command, run_command,
};

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Rdm Error:"]
#[doc_to_pretty]
pub(crate) enum RdmError {
    ConfigError(config::ConfigError),
    ConfigCommandError(config_command::ConfigCommandError),
    InitCommandError(init_command::InitError),
    BootstrapError(bootstrap_command::BootstrapError),
    RunCommandError(run_command::RunCommandError),
}
