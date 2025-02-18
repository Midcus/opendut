use std::sync::{Arc, Mutex};

use opendut_types::peer::{self, executor::{ExecutorDescriptor, ExecutorKind}};
use tokio::sync::watch::{self, Sender};
use tracing::warn;

use crate::service::test_execution::container_manager::{ContainerManager, ContainerConfiguration};

pub type ExecutorManagerRef = Arc<Mutex<ExecutorManager>>;

pub struct ExecutorManager {
    tx_termination_channels: Vec<Sender<bool>>,
}

impl ExecutorManager {
    pub fn create() -> ExecutorManagerRef {
        Arc::new(Mutex::new(Self {
            tx_termination_channels: Vec::new(),
        }))
    }

    pub fn create_new_executors(&mut self, executors: Vec<peer::configuration::Parameter<ExecutorDescriptor>>) {

        let executors = executors.into_iter()
            .filter_map(|executor| { //TODO properly handle Present vs. Absent
                if matches!(executor.target, peer::configuration::ParameterTarget::Present) {
                    Some(executor.value)
                } else {
                    None
                }
            });

        for executor in executors {

            let (tx, rx) = watch::channel(false);

            let ExecutorDescriptor {kind, results_url} = executor;

            match kind {
                ExecutorKind::Executable => warn!("Executing Executable not yet implemented."),
                ExecutorKind::Container {
                    engine,
                    name,
                    image,
                    volumes,
                    devices,
                    envs,
                    ports,
                    command,
                    args,
                } => {
                    let container_config = ContainerConfiguration{
                        name,
                        engine,
                        image,
                        command,
                        args,
                        envs,
                        results_url,
                        ports,
                        devices,
                        volumes,
                    };
                    tokio::spawn(async move {
                        ContainerManager::new(container_config, rx).start().await;
                    });
                }
            }
            self.tx_termination_channels.push(tx);
        }
    }

    pub fn terminate_executors(&mut self) {
        for tx_termination_channel in &self.tx_termination_channels {
            if let Err(cause) = tx_termination_channel.send(true) {
                warn!("Failed to send termination signal to executor, perhaps it already terminated? Cause: {cause}");
            }
        }
        self.tx_termination_channels.clear();

    }


}
