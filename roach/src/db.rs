use async_trait::async_trait;
use crate::player::Player;
use crate::hive_match::HiveMatch;
use std::collections::HashMap;
use tokio::sync::{RwLock};
use std::sync::Arc;
use serde::Serialize;

#[derive(Serialize)]
pub struct DBError;

pub enum Range {
    All,
    StartLimit(usize, usize),
}

#[async_trait]
pub trait DB<T> {
    async fn find(&self, id: String) -> Option<T>;
    async fn get(&self, range: Range) -> Vec<T>;
    async fn create(&mut self, item: T) -> Result<T, DBError>;
    async fn update(&mut self, item: T) -> Result<T, DBError>;
    async fn delete(&mut self, id: String) -> Result<(), DBError>;
}

pub struct MockDB<T> {
    map: HashMap<String, T>,
}

pub trait Id {
    fn id(&self) -> String;
}

impl Id for Player {
    fn id(&self) -> String { self.name.clone() }
}

pub type PlayerDB = Arc<RwLock<MockDB<Player>>>;

impl<T> MockDB<T> {
    pub fn new() -> MockDB<T> {
        MockDB { map: HashMap::new() }
    }
}

#[async_trait]
impl<T> DB<T> for MockDB<T> where T: Id + Clone + Sync + Send {
    async fn find(&self, id: String) -> Option<T> {
        self.map.get(&id).cloned()
    }
    async fn get(&self, range: Range) -> Vec<T> {
        self.map.values().cloned().collect()
    }
    async fn create(&mut self, item: T) -> Result<T, DBError> {
        self.map.insert(item.id(), item.clone()).ok_or(DBError)
    }
    async fn update(&mut self, updated_item: T) -> Result<T, DBError> {
        self.map.entry(updated_item.id()).and_modify(|e| *e = updated_item.clone());
        Ok(updated_item)
    }
    async fn delete(&mut self, id: String) -> Result<(), DBError> {
        self.map.remove(&id);
        Ok(())
    }
}
