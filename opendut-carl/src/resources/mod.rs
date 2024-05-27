use std::any::Any;
use std::collections::hash_map::{Values, ValuesMut};
use std::collections::HashMap;
use std::marker::PhantomData;

use opendut_types::resources::Id;

use crate::resources::storage::{ResourcesStorage, ResourcesStorageApi, ResourcesStorageOptions};

pub mod manager;
pub mod ids;
pub(crate) mod storage;
mod resource;

use ids::IntoId;
use resource::Resource;

pub struct Resources {
    storage: ResourcesStorage,
}

impl Resources {
    pub fn new(storage_options: ResourcesStorageOptions) -> Self {
        let storage = ResourcesStorage::new(storage_options);
        Self { storage }
    }

    pub fn insert<R>(&mut self, id: impl IntoId<R>, resource: R)
    where R: Resource {
        match &mut self.storage {
            ResourcesStorage::Database(storage) => storage.insert(id, resource),
            ResourcesStorage::Memory(storage) => storage.insert(id, resource),
        };
    }

    pub fn update<R>(&mut self, id: impl IntoId<R>) -> Update<R>
    where R: Resource {
        match &mut self.storage {
            ResourcesStorage::Database(storage) => storage.update(id),
            ResourcesStorage::Memory(storage) => storage.update(id),
        }
    }

    pub fn remove<R>(&mut self, id: impl IntoId<R>) -> Option<R>
    where R: Resource {
        match &mut self.storage {
            ResourcesStorage::Database(storage) => storage.remove(id),
            ResourcesStorage::Memory(storage) => storage.remove(id),
        }
    }

    pub fn get<R>(&self, id: impl IntoId<R>) -> Option<R>
    where R: Resource + Clone {
        match &self.storage {
            ResourcesStorage::Database(storage) => storage.get(id),
            ResourcesStorage::Memory(storage) => storage.get(id),
        }
    }

    pub fn iter<R>(&self) -> Iter<R>
    where R: Resource {
        match &self.storage {
            ResourcesStorage::Database(storage) => storage.iter(),
            ResourcesStorage::Memory(storage) => storage.iter(),
        }
    }

    pub fn iter_mut<R>(&mut self) -> IterMut<R>
    where R: Resource {
        match &mut self.storage {
            ResourcesStorage::Database(storage) => storage.iter_mut(),
            ResourcesStorage::Memory(storage) => storage.iter_mut(),
        }
    }
}

#[cfg(test)]
impl Resources {
    pub async fn contains<R>(&self, id: impl IntoId<R>) -> bool
    where R: Resource + Clone {
        match &self.storage {
            ResourcesStorage::Database(_) => unimplemented!(),
            ResourcesStorage::Memory(storage) => storage.contains(id),
        }
    }

    pub async fn is_empty(&self) -> bool {
        match &self.storage {
            ResourcesStorage::Database(_) => unimplemented!(),
            ResourcesStorage::Memory(storage) => storage.is_empty(),
        }
    }
}


pub struct Update<'a, R>
where R: Any + Send + Sync {
    id: Id,
    column: &'a mut HashMap<Id, Box<dyn Any + Send + Sync>>,
    marker: PhantomData<R>,
}

impl <R> Update<'_, R>
where R: Any + Send + Sync {

    pub fn modify<F>(self, f: F) -> Self
    where F: FnOnce(&mut R) {
        if let Some(boxed) = self.column.get_mut(&self.id) {
            if let Some(resource) = boxed.downcast_mut() {
                f(resource)
            }
        }
        self
    }

    pub fn or_insert(self, resource: R) {
        self.column.entry(self.id).or_insert_with(|| Box::new(resource));
    }
}

pub struct Iter<'a, R>
where R: Resource {
    column: Option<Values<'a, Id, Box<dyn Any + Send + Sync>>>,
    marker: PhantomData<R>
}

impl <'a, R> Iter<'a, R>
where R: Resource {
    fn new(column: Option<Values<'a, Id, Box<dyn Any + Send + Sync>>>) -> Iter<'a, R> {
        Self {
            column,
            marker: PhantomData
        }
    }
}

impl <'a, R> Iterator for Iter<'a, R>
where R: Resource {

    type Item = &'a R;

    fn next(&mut self) -> Option<Self::Item> {
        let column = self.column.as_mut()?;
        column.next()
            .and_then(|value| value.downcast_ref())
    }
}


pub struct IterMut<'a, R>
where R: Resource {
    column: Option<ValuesMut<'a, Id, Box<dyn Any + Send + Sync>>>,
    marker: PhantomData<R>
}

impl <'a, R> IterMut<'a, R>
where R: Resource {
    fn new(column: Option<ValuesMut<'a, Id, Box<dyn Any + Send + Sync>>>) -> IterMut<'a, R> {
        Self {
            column,
            marker: PhantomData
        }
    }
}

impl <'a, R> Iterator for IterMut<'a, R>
where R: Resource {

    type Item = &'a mut R;

    fn next(&mut self) -> Option<Self::Item> {
        let column = self.column.as_mut()?;
        column.next()
            .and_then(|value| value.downcast_mut())
    }
}
