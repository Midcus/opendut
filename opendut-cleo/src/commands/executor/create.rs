use opendut_types::peer::executor::ExecutorDescriptor;
use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;
use opendut_types::peer::executor::{container::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerEnvironmentVariable, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume, Engine}, ExecutorKind, ResultsUrl};

use crate::{CreateOutputFormat, DescribeOutputFormat, EngineVariants};

/// Create a container executor using command-line arguments
#[derive(clap::Parser)]
pub struct CreateContainerExecutorCli {
    ///ID of the peer to add the container executor to
    #[arg(long)]
    peer_id: Uuid,
    ///Engine
    #[arg(short, long)]
    engine: EngineVariants,
    ///Container name
    #[arg(short, long)]
    name: Option<ContainerName>,
    ///Container image
    #[arg(short, long)]
    image: ContainerImage,
    ///Container volumes
    #[arg(short, long, num_args = 1..)]
    volumes: Option<Vec<ContainerVolume>>,
    ///Container devices
    #[arg(long, num_args = 1..)]
    devices: Option<Vec<ContainerDevice>>,
    ///Container envs
    #[arg(long, num_args = 1..)]
    envs: Option<Vec<String>>,
    ///Container ports
    #[arg(short, long, num_args = 1..)]
    ports: Option<Vec<ContainerPortSpec>>,
    ///Container command
    #[arg(short, long)]
    command: Option<ContainerCommand>,
    ///Container arguments
    #[arg(short, long, num_args = 1..)]
    args: Option<Vec<ContainerCommandArgument>>,
    ///URL to which results will be uploaded
    #[arg(short, long)]
    results_url: Option<ResultsUrl>,
}

impl CreateContainerExecutorCli {
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(self, carl: &mut CarlClient, output: CreateOutputFormat) -> crate::Result<()> {

        let engine = match self.engine {
            EngineVariants::Docker => { Engine::Docker }
            EngineVariants::Podman => { Engine::Podman }
        };

        let volumes = self.volumes.unwrap_or_default();
        let devices = self.devices.unwrap_or_default();
        let ports = self.ports.unwrap_or_default();
        let args = self.args.unwrap_or_default();

        let mut environment_variables = vec![];

        for env in self.envs.unwrap_or_default() {
            if let Some((name, value)) = env.split_once('=') {
                let env = ContainerEnvironmentVariable::new(name, value)
                    .map_err(|cause| cause.to_string())?;
                environment_variables.push(env)
            }
        };

        let executor_descriptor = ExecutorDescriptor {
            kind: ExecutorKind::Container {
                engine,
                name: self.name.unwrap_or_default(),
                image: self.image,
                volumes,
                devices,
                envs: environment_variables,
                ports,
                command: self.command.unwrap_or_default(),
                args,
            },
            results_url: self.results_url,
        };

        let peer_id = PeerId::from(self.peer_id);
        

        let mut peer_descriptor = carl.peers.get_peer_descriptor(peer_id).await
            .map_err(|_| format!("Failed to get peer with ID <{}>.", peer_id))?;

        peer_descriptor.executors.executors.push(executor_descriptor);

        carl.peers.store_peer_descriptor(Clone::clone(&peer_descriptor)).await
            .map_err(|error| format!("Failed to update peer <{}>.\n  {}", peer_id, error))?;
        let output_format = DescribeOutputFormat::from(output);
        crate::commands::peer::describe::render_peer_descriptor(peer_descriptor, output_format);

        Ok(())
    }
}
