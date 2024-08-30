use std::sync::Arc;

use crate::persistence::error::{FlattenPersistenceResult, PersistenceResult};
use crate::persistence::resources::Persistable;
use crate::resources::storage::{PersistenceOptions, ResourcesStorageApi};
use crate::resources::{storage, Resource, Resources, ResourcesTransaction};
use tokio::sync::RwLock;

pub type ResourcesManagerRef = Arc<ResourcesManager>;

pub struct ResourcesManager {
    state: RwLock<State>,
}

struct State {
    resources: Resources
}

impl ResourcesManager {

    pub async fn create(storage_options: PersistenceOptions) -> Result<ResourcesManagerRef, storage::ConnectionError> {
        let resources = Resources::connect(storage_options).await?;

        Ok(Arc::new(Self {
            state: RwLock::new(State { resources })
        }))
    }

    pub async fn insert<R>(&self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        let mut state = self.state.write().await;
        state.resources.transaction(move |mut transaction| {
            transaction.insert(id, resource)
        }).flatten_persistence_result()
    }

    pub async fn remove<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        let mut state = self.state.write().await;
        state.resources.transaction(move |mut transaction| {
            transaction.remove(id)
        }).flatten_persistence_result()
    }

    pub async fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        let state = self.state.read().await;
        state.resources.get(id)
    }

    pub async fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where R: Resource + Persistable + Clone {
        let state = self.state.read().await;
        state.resources.list()
    }

    pub async fn resources<F, T>(&self, f: F) -> PersistenceResult<T>
    where F: FnOnce(&Resources) -> PersistenceResult<T> {
        let state = self.state.read().await;
        f(&state.resources)
    }

    /// Allows grouping modifications to the database. This does multiple things:
    /// - Opens a database transaction and then either commits it, or rolls it back when you return an `Err` out of the closure.
    /// - Acquires the lock for the database mutex and keeps it until the end of the closure.
    /// - Groups the async calls, so we only have to await at the end.
    pub async fn resources_mut<F, T, E>(&self, f: F) -> PersistenceResult<Result<T, E>>
    where
        F: FnOnce(&mut ResourcesTransaction) -> Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut state = self.state.write().await;
        state.resources.transaction(move |mut transaction| {
            f(&mut transaction)
        })
    }
}

#[cfg(test)]
impl ResourcesManager {
    pub fn new_in_memory() -> ResourcesManagerRef {
        let resources = futures::executor::block_on(
            Resources::connect(PersistenceOptions::Disabled)
        )
        .expect("Creating in-memory storage for tests should not fail");

        Arc::new(Self {
            state: RwLock::new(State { resources })
        })
    }

    async fn contains<R>(&self, id: R::Id) -> bool
    where R: Resource + Clone {
        let state = self.state.read().await;
        state.resources.contains::<R>(id).await
    }

    async fn is_empty(&self) -> bool {
        let state = self.state.read().await;
        state.resources.is_empty().await
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::ops::Not;
    use std::vec;

    use googletest::prelude::*;

    use super::*;
    use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
    use opendut_types::peer::executor::{container::{ContainerCommand, ContainerImage, ContainerName, Engine}, ExecutorDescriptor, ExecutorDescriptors, ExecutorId, ExecutorKind};
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::topology::Topology;
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    #[tokio::test]
    async fn test() -> Result<()> {

        let testee = ResourcesManager::new_in_memory();

        let peer_resource_id = PeerId::random();
        let peer = PeerDescriptor {
            id: peer_resource_id,
            name: PeerName::try_from("TestPeer").unwrap(),
            location: PeerLocation::try_from("Ulm").ok(),
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        id: NetworkInterfaceId::random(),
                        name: NetworkInterfaceName::try_from("eth0").unwrap(),
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                ],
                bridge_name: Some(NetworkInterfaceName::try_from("br-opendut-1").unwrap()),
            },
            topology: Topology::default(),
            executors: ExecutorDescriptors {
                executors: vec![
                    ExecutorDescriptor {
                        id: ExecutorId::random(),
                        kind: ExecutorKind::Container {
                            engine: Engine::Docker,
                            name: ContainerName::Empty,
                            image: ContainerImage::try_from("testUrl").unwrap(),
                            volumes: vec![],
                            devices: vec![],
                            envs: vec![],
                            ports: vec![],
                            command: ContainerCommand::Default,
                            args: vec![],
                        },
                        results_url: None,
                    }
                ],
            }
        };

        let cluster_resource_id = ClusterId::random();
        let cluster_configuration = ClusterConfiguration {
            id: cluster_resource_id,
            name: ClusterName::try_from("ClusterX032").unwrap(),
            leader: peer.id,
            devices: HashSet::new(),
        };

        assert!(testee.is_empty().await);

        testee.insert(peer_resource_id, Clone::clone(&peer)).await?;

        assert!(testee.is_empty().await.not());

        testee.insert(cluster_resource_id, Clone::clone(&cluster_configuration)).await?;

        assert_that!(testee.get::<PeerDescriptor>(peer_resource_id).await?, some(eq(Clone::clone(&peer))));
        assert_that!(testee.get::<ClusterConfiguration>(cluster_resource_id).await?, some(eq(Clone::clone(&cluster_configuration))));

        assert!(testee.contains::<PeerDescriptor>(peer_resource_id).await);

        assert_that!(testee.get::<PeerDescriptor>(PeerId::random()).await?, none());

        assert_that!(testee.remove::<PeerDescriptor>(peer_resource_id).await?, some(eq(Clone::clone(&peer))));

        testee.insert(peer_resource_id, Clone::clone(&peer)).await?;

        assert_that!(testee.get::<PeerDescriptor>(peer_resource_id).await?, some(eq(Clone::clone(&peer))));

        testee.resources(|resources| {
            resources.list::<ClusterConfiguration>()?
                .into_iter()
                .for_each(|cluster| {
                    assert_that!(cluster, eq(Clone::clone(&cluster_configuration)));
                });
            Ok(())
        }).await?;

        Ok(())
    }
}
