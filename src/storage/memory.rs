use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use log::warn;

use crate::resp::value::Value;
use super::entities::Entities;

#[derive(Clone)]
pub struct MemoryStore {
  general_store: Arc<Mutex<HashMap<String, Value>>>,
  acl_store: Arc<Mutex<HashMap<String, HashMap<String, Entities>>>>,
  authenticated: Arc<Mutex<bool>>,
}

pub trait Store {
  fn new() -> Self;

  async fn set(&self, key: &str, value: Value);
  async fn get(&self, key: &str) -> Option<Value>;
  async fn delete(&self, key: &str) -> Option<Value>;
}

impl Store for MemoryStore {
  fn new() -> Self {
    // @INFO Initialize the memory store
    let acl_store = Arc::new(Mutex::new(HashMap::new()));
    let general_store = Arc::new(Mutex::new(HashMap::new()));
    let authenticated = Arc::new(Mutex::new(false));

    // @INFO put the predefined entities inside the stores

    Self {
      general_store,
      acl_store,
      authenticated,
    }
  }

  async fn set(&self, key: &str, value: Value) {
    if self.authenticated.lock().unwrap().eq(&false) {
      warn!("User not authenticated, using shared storage");
      let mut db = self.general_store.lock().unwrap();
      if db.contains_key(key) {
        warn!("Key {} already exists, overwriting value", key);
        db.insert(key.to_string(), value);
      } else {
        db.insert(key.to_string(), value);
      }
    } else {
      // @TODO handle acl storage
      todo!()
    }
  }

  async fn get(&self, key: &str) -> Option<Value> {
    if self.authenticated.lock().unwrap().eq(&false) {
      warn!("User not authenticated, using shared storage");
      let db = self.general_store.lock().unwrap();
      match db.get(key) {
        Some(value) => Some(value.clone()),
        None => Some(Value::Null),
      }
    } else {
      // @TODO handle acl storage
      todo!()
    }
  }

  async fn delete(&self, key: &str) -> Option<Value> {
    if self.authenticated.lock().unwrap().eq(&false) {
      warn!("User not authenticated, using shared storage");
      let mut db = self.general_store.lock().unwrap();
      let result = db.remove(key);
      if result.is_none() {
        warn!("Key {} not found, cannot delete", key);
      }
      result
    } else {
      // @TODO handle acl storage
      todo!()
    }
  }
}
