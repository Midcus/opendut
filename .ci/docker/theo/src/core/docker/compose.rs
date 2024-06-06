use anyhow::Error;

use crate::core::docker::determine_if_ports_shall_be_exposed;
use crate::core::docker::command::DockerCommand;
use crate::core::docker::services::DockerCoreServices;

pub(crate) fn docker_compose_build(compose_dir: &str) -> Result<i32, Error> {
    DockerCommand::new()
        .add_common_args(compose_dir)
        .arg("build")
        // https://docs.docker.com/build/building/env-vars/
        // Show more output during the build progress
        .env("BUILDKIT_PROGRESS", "plain")
        .expect_status(format!("Failed to execute docker compose build for directory: {}.", compose_dir).as_str())
}

pub fn docker_compose_up_expose_ports(compose_dir: &str, expose: bool) -> crate::Result {
    let mut command = DockerCommand::new();
    command.arg("compose")
        .arg("--file")
        .arg(format!(".ci/docker/{}/docker-compose.yml", compose_dir));

    if determine_if_ports_shall_be_exposed(expose) {
        command.arg("--file")
            .arg(format!(".ci/docker/{}/expose_ports.yml", compose_dir))
    } else {
        command.arg("--file")
            .arg(format!(".ci/docker/{}/localhost_ports.yml", compose_dir))
    };
    command.arg("--env-file")
        .arg(".env-theo")
        .arg("--env-file")
        .arg(".env")
        .arg("up")
        .arg("--detach")
        .expect_status(&format!("Failed to execute docker compose command for {}.", compose_dir))?;
    Ok(())
}


pub(crate) fn docker_compose_down(compose_dir: &str, delete_volumes: bool) -> Result<i32, Error> {
    let mut command = DockerCommand::new();
    command.add_common_args(compose_dir);
    if delete_volumes {
        command.arg("down").arg("--volumes");
    } else {
        command.arg("down");
    }
    command.expect_status(format!("Failed to execute docker compose down for directory: {}.", compose_dir).as_str())
}

pub(crate) fn docker_compose_network_create() -> Result<i32, Error> {
    DockerCommand::new()
        .arg("compose")
        .arg("--file")
        .arg(format!("./.ci/docker/{}/docker-compose.yml", DockerCoreServices::Network))
        .arg("up")
        .arg("--force-recreate")
        .expect_status("Failed to create docker network.")
}

pub(crate) fn docker_compose_network_delete() -> Result<i32, Error> {
    DockerCommand::new()
        .arg("network")
        .arg("rm")
        .arg("opendut_network")
        .expect_status("Failed to delete docker network.")
}
