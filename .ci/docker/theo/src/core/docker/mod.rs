use std::env;

use anyhow::Error;

use command::DockerCommand;

use crate::core::{OPENDUT_EXPOSE_PORTS, TheoError};

mod checks;
pub(crate) mod compose;

pub(crate) mod keycloak;
pub(crate) mod netbird;
pub(crate) mod edgar;
pub(crate) mod command;
pub(crate) mod services;

/// The environment variable is managed by THEO.
/// Within the VM it is set to true, outside the VM it defaults to false.
/// Be careful when overriding this value.
pub fn determine_if_ports_shall_be_exposed(user_intents_to_expose: bool) -> bool {
    let expose_env_value = env::var(OPENDUT_EXPOSE_PORTS).unwrap_or("false".to_string()).eq("true");
    user_intents_to_expose || expose_env_value
}


pub fn show_error_if_unhealthy_containers_were_found() -> Result<(), Error> {
    let unhealthy_containers = DockerCommand::enumerate_unhealthy_containers()?;
    if unhealthy_containers.is_empty() {
        println!("# No unhealthy containers found.");
        Ok(())
    } else {
        println!("# Unhealthy containers: {:?}", unhealthy_containers);
        Err(TheoError::UnhealthyContainersFound(format!("Found unhealthy docker containers: {:?}", unhealthy_containers)).into())
    }
}

pub fn start_netbird(expose: bool) -> Result<i32, Error> {
    let mut command = DockerCommand::new();
    command.arg("compose")
        .arg("--file")
        .arg(".ci/docker/netbird/docker-compose.yml");

    if determine_if_ports_shall_be_exposed(expose) {
        command.arg("--file")
            .arg(".ci/docker/netbird/expose_ports.yml");
    };
    command.arg("--env-file")
        .arg(".env-theo")
        .arg("--env-file")
        .arg(".env")
        .arg("up")
        .arg("--detach")
        .expect_status("Failed to execute compose command for netbird")
}



