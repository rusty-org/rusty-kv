use std::{
  collections::HashMap,
  sync::{Arc, Mutex, RwLock},
};

use log::{info, warn};

use crate::resp::value::Value;
use super::entities::Entities;

#[derive(Clone)]
pub struct MemoryStore {
  // Global store for all users data, keyed by user credential hash
  user_stores: Arc<RwLock<HashMap<String, UserStore>>>,
  // Current user's credential hash (if authenticated)
  current_user: Arc<RwLock<Option<String>>>,
}

// Represents a single user's data store
#[derive(Clone, Debug)]
pub struct UserStore {
  general_store: Arc<Mutex<HashMap<String, Value>>>,
  entities_store: Arc<Mutex<HashMap<String, Entities>>>,
}

impl UserStore {
  fn new() -> Self {
    Self {
      general_store: Arc::new(Mutex::new(HashMap::new())),
      entities_store: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

pub trait Store {
  fn new() -> Self;

  async fn set(&self, key: &str, value: Value);
  async fn get(&self, key: &str) -> Option<Value>;
  async fn delete(&self, key: &str) -> Option<Value>;

  // New methods for authentication
  fn set_current_user(&self, user_hash: Option<String>);
  fn get_current_user(&self) -> Option<String>;
  fn is_authenticated(&self) -> bool;

  // Entity methods
  async fn create_entity(&self, entity_type: &str, name: &str) -> anyhow::Result<()>;
  async fn entity_add(&self, entity_name: &str, key: &str, value: Value) -> anyhow::Result<()>;
  async fn entity_get(&self, entity_name: &str, key: &str) -> anyhow::Result<Option<Value>>;
  async fn entity_delete(&self, entity_name: &str, key: &str) -> anyhow::Result<Option<Value>>;
}

impl Store for MemoryStore {
  fn new() -> Self {
    info!("Initializing global memory store");
    Self {
      user_stores: Arc::new(RwLock::new(HashMap::new())),
      current_user: Arc::new(RwLock::new(None)),
    }
  }

  fn set_current_user(&self, user_hash: Option<String>) {
    let mut current_user = self.current_user.write().unwrap();
    *current_user = user_hash;

    // Initialize user store if it doesn't exist
    if let Some(hash) = current_user.clone() {
      let mut stores = self.user_stores.write().unwrap();
      if !stores.contains_key(&hash) {
        info!("Creating new store for user with hash: {}", hash);
        stores.insert(hash, UserStore::new());
      }
    }
  }

  fn get_current_user(&self) -> Option<String> {
    self.current_user.read().unwrap().clone()
  }

  fn is_authenticated(&self) -> bool {
    self.current_user.read().unwrap().is_some()
  }

  async fn set(&self, key: &str, value: Value) {
    if let Some(user_hash) = self.get_current_user() {
      let stores = self.user_stores.read().unwrap();
      if let Some(user_store) = stores.get(&user_hash) {
        let mut store = user_store.general_store.lock().unwrap();
        store.insert(key.to_string(), value);
        info!("Set key '{}' for authenticated user", key);
        return;
      }
    }

    // Fallback to shared store for unauthenticated users
    warn!("User not authenticated, using shared storage");
    let mut stores = self.user_stores.write().unwrap();
    let shared_store = stores.entry("shared".to_string()).or_insert_with(UserStore::new);
    let mut store = shared_store.general_store.lock().unwrap();
    store.insert(key.to_string(), value);
  }

  async fn get(&self, key: &str) -> Option<Value> {
    if let Some(user_hash) = self.get_current_user() {
      let stores = self.user_stores.read().unwrap();
      if let Some(user_store) = stores.get(&user_hash) {
        let store = user_store.general_store.lock().unwrap();
        if let Some(value) = store.get(key) {
          return Some(value.clone());
        }
      }
    }

    // Try shared store if not found in user store
    let stores = self.user_stores.read().unwrap();
    if let Some(shared_store) = stores.get("shared") {
      let store = shared_store.general_store.lock().unwrap();
      return store.get(key).cloned().or(Some(Value::Null));
    }

    Some(Value::Null)
  }

  async fn delete(&self, key: &str) -> Option<Value> {
    if let Some(user_hash) = self.get_current_user() {
      let stores = self.user_stores.read().unwrap();
      if let Some(user_store) = stores.get(&user_hash) {
        let mut store = user_store.general_store.lock().unwrap();
        return store.remove(key);
      }
    }

    // Try shared store
    let stores = self.user_stores.read().unwrap();
    if let Some(shared_store) = stores.get("shared") {
      let mut store = shared_store.general_store.lock().unwrap();
      return store.remove(key);
    }

    None
  }

  async fn create_entity(&self, entity_type: &str, name: &str) -> anyhow::Result<()> {
    let user_hash = self.get_current_user().unwrap_or_else(|| "shared".to_string());
    let stores = self.user_stores.read().unwrap();

    let user_store = stores.get(&user_hash).unwrap_or_else(|| {
      panic!("User store not found for hash: {}", user_hash)
    });

    let mut entities = user_store.entities_store.lock().unwrap();

    let entity = match entity_type.to_lowercase().as_str() {
      "set" => Entities::Set(Arc::new(Mutex::new(super::entities::KvSet::new()))),
      "hashmap" => Entities::HashMap(Arc::new(Mutex::new(super::entities::KvHashMap::new()))),
      _ => return Err(anyhow::anyhow!("Unknown entity type: {}", entity_type)),
    };

    entities.insert(name.to_string(), entity);
    Ok(())
  }

  async fn entity_add(&self, entity_name: &str, key: &str, value: Value) -> anyhow::Result<()> {
    let user_hash = self.get_current_user().unwrap_or_else(|| "shared".to_string());
    let stores = self.user_stores.read().unwrap();

    let user_store = stores.get(&user_hash).unwrap_or_else(|| {
      panic!("User store not found for hash: {}", user_hash)
    });

    let entities = user_store.entities_store.lock().unwrap();

    if let Some(entity) = entities.get(entity_name) {
      match entity {
        Entities::Set(set) => {
          let mut set = set.lock().unwrap();
          // For HashSet, we insert the key as a string (no value needed)
          if let Value::SimpleString(val) = &value {
            set.insert(val.clone());
          } else {
            set.insert(key.to_string());
          }
        },
        Entities::HashMap(hashmap) => {
          let mut hashmap = hashmap.lock().unwrap();
          hashmap.insert(key.to_string(), value);
        },
        _ => return Err(anyhow::anyhow!("Entity type not supported for this operation")),
      }
      Ok(())
    } else {
      Err(anyhow::anyhow!("Entity not found: {}", entity_name))
    }
  }

  async fn entity_get(&self, entity_name: &str, key: &str) -> anyhow::Result<Option<Value>> {
    let user_hash = self.get_current_user().unwrap_or_else(|| "shared".to_string());
    let stores = self.user_stores.read().unwrap();

    let user_store = stores.get(&user_hash).unwrap_or_else(|| {
      panic!("User store not found for hash: {}", user_hash)
    });

    let entities = user_store.entities_store.lock().unwrap();

    if let Some(entity) = entities.get(entity_name) {
      match entity {
        Entities::Set(set) => {
          let set = set.lock().unwrap();
          if set.contains(key) {
            Ok(Some(Value::SimpleString(key.to_string())))
          } else {
            Ok(None)
          }
        },
        Entities::HashMap(hashmap) => {
          let hashmap = hashmap.lock().unwrap();
          Ok(hashmap.get(key).cloned())
        },
        _ => Err(anyhow::anyhow!("Entity type not supported for this operation")),
      }
    } else {
      Err(anyhow::anyhow!("Entity not found: {}", entity_name))
    }
  }

  async fn entity_delete(&self, entity_name: &str, key: &str) -> anyhow::Result<Option<Value>> {
    let user_hash = self.get_current_user().unwrap_or_else(|| "shared".to_string());
    let stores = self.user_stores.read().unwrap();

    let user_store = stores.get(&user_hash).unwrap_or_else(|| {
      panic!("User store not found for hash: {}", user_hash)
    });

    let entities = user_store.entities_store.lock().unwrap();

    if let Some(entity) = entities.get(entity_name) {
      match entity {
        Entities::Set(set) => {
          let mut set = set.lock().unwrap();
          let removed = set.remove(key);
          if removed {
            Ok(Some(Value::SimpleString(key.to_string())))
          } else {
            Ok(None)
          }
        },
        Entities::HashMap(hashmap) => {
          let mut hashmap = hashmap.lock().unwrap();
          Ok(hashmap.remove(key))
        },
        _ => Err(anyhow::anyhow!("Entity type not supported for this operation")),
      }
    } else {
      Err(anyhow::anyhow!("Entity not found: {}", entity_name))
    }
  }
}
