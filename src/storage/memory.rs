use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use log::warn;

use crate::resp::value::Value;

#[derive(Clone)]
pub struct MemoryStore {
  db: Arc<Mutex<HashMap<String, Value>>>,
}

pub trait Store {
  fn new() -> Self;

  async fn set(&self, key: &str, value: Value);
  async fn get(&self, key: &str) -> Option<Value>;
  async fn delete(&self, key: &str) -> Option<Value>;
}

impl Store for MemoryStore {
  fn new() -> Self {
    Self {
      db: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  async fn set(&self, key: &str, value: Value) {
    let mut db = self.db.lock().unwrap();
    if db.contains_key(key) {
      warn!("Key {} already exists, overwriting value", key);
      db.insert(key.to_string(), value);
    } else {
      db.insert(key.to_string(), value);
    }
  }

  async fn get(&self, key: &str) -> Option<Value> {
    let db = self.db.lock().unwrap();
    match db.get(key) {
      Some(value) => Some(value.clone()),
      None => Some(Value::Null),
    }
  }

  async fn delete(&self, key: &str) -> Option<Value> {
    let mut db = self.db.lock().unwrap();
    let result = db.remove(key);
    if result.is_none() {
      warn!("Key {} not found, cannot delete", key);
    }
    result
  }
}
