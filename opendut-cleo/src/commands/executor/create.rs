use std::path::Path;
use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;
use opendut_types::peer::executor::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerEnvironmentVariable, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume, Engine, ExecutorDescriptor, ResultsUrl};
use crate::{CreateOutputFormat, DescribeOutputFormat, EngineVariants};

/// Create a container executor
#[derive(clap::Parser)]
pub struct CreateContainerExecutorCli {
    #[clap(subcommand)]
    pub mode: Mode,
}

#[derive(clap::Parser)]
pub enum Mode{
    JsonConfig(JsonConfig),
    CommandLineArguments(CommandLineArguments),
}

#[derive(clap::Parser)]
pub struct JsonConfig {
    #[clap(short, long, default_value = "sample_test_config.json", required = true, help = "File path to the JSON test execution configuration")]
    test_executor_json_file_path: String,
}

impl AsRef<Path> for JsonConfig {
    fn as_ref(&self) -> &Path {
        Path::new(&self.test_executor_json_file_path)
    }
}

#[derive(clap::Parser)]
pub struct CommandLineArguments {
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
}

impl CreateContainerExecutorCli {
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(self, carl: &mut CarlClient, output: CreateOutputFormat) -> crate::Result<()> {
        let peer_id;
        let executor_descriptor:ExecutorDescriptor; 
        match self.mode {
            Mode::JsonConfig(json_config) => {
                println!("File path{:?}", json_config.test_executor_json_file_path);
                let test_executor_config = std::fs::read_to_string(json_config.test_executor_json_file_path).map_err(|e| e.to_string())?;
                
                executor_descriptor = match
                    serde_json::from_str(&test_executor_config) {
                        Ok(map) => { map }
                        Err(error) => {
                            return Err("Error parsing JSON: ".to_string() + &error.to_string());
                        }
                    };
                
                match &executor_descriptor {
                    ExecutorDescriptor::Container { preconditions, .. } => {
                        peer_id = PeerId::from(preconditions.device_preconditions[0].device_id());
                    },
                    ExecutorDescriptor::Executable => {
                        return Err("Expected a Container ExecutorDescriptor, but got Executable".to_string());
                    }
                }
                
            },
            Mode::CommandLineArguments(cmd_args)=> {

                peer_id = PeerId::from(cmd_args.peer_id);

                let engine = match cmd_args.engine {
                    EngineVariants::Docker => { Engine::Docker }
                    EngineVariants::Podman => { Engine::Podman }
                };

                let volumes = cmd_args.volumes.unwrap_or_default();
                let devices = cmd_args.devices.unwrap_or_default();
                let ports = cmd_args.ports.unwrap_or_default();
                let args = cmd_args.args.unwrap_or_default();

                let mut environment_variables = vec![];

                for env in cmd_args.envs.unwrap_or_default() {
                    if let Some((name, value)) = env.split_once('=') {
                        let env = ContainerEnvironmentVariable::new(name, value)
                            .map_err(|cause| cause.to_string())?;
                        environment_variables.push(env)
                    }
                };

                executor_descriptor = ExecutorDescriptor::Container {
                    engine,
                    name: cmd_args.name.unwrap_or_default(),
                    image: cmd_args.image,
                    volumes,
                    devices,
                    envs: environment_variables,
                    ports,
                    command: cmd_args.command.unwrap_or_default(),
                    args,
                    preconditions: Default::default(),
                    results_url: ResultsUrl::default(),
                };
            }
        }

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
