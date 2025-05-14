use std::{
  collections::HashMap,
  sync::{Arc, Mutex, RwLock},
};

use log::info;

use crate::resp::value::Value;
use super::entities::Entities;

#[derive(Clone)]
pub struct MemoryStore {
  // Store for authenticated users, keyed by user credential hash
  auth_stores: Arc<RwLock<HashMap<String, UserStore>>>,
  // Store for unauthenticated sessions (shared)
  unauth_store: Arc<RwLock<UserStore>>,
  // Current user's credential hash (if authenticated)
  current_user: Arc<RwLock<Option<String>>>,
}

// Represents a single user's data store
#[derive(Clone, Debug)]
pub struct UserStore {
  // Stores entity references for various data types
  // Key is entity name, value is the entity (HashMap, Set, etc)
  entities: Arc<Mutex<HashMap<String, Entities>>>,
}

impl UserStore {
  fn new() -> Self {
    Self {
      entities: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

pub trait Store {
  fn new() -> Self;

  async fn set(&self, key: &str, value: Value) -> anyhow::Result<()>;
  async fn get(&self, key: &str) -> Option<Value>;
  async fn delete(&self, key: &str) -> Option<Value>;

  // Authentication methods
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
      auth_stores: Arc::new(RwLock::new(HashMap::new())),
      unauth_store: Arc::new(RwLock::new(UserStore::new())),
      current_user: Arc::new(RwLock::new(None)),
    }
  }

  fn set_current_user(&self, user_hash: Option<String>) {
    let mut current_user = self.current_user.write().unwrap();
    *current_user = user_hash;

    // Initialize user store if it doesn't exist
    if let Some(hash) = current_user.clone() {
      let mut stores = self.auth_stores.write().unwrap();
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

  async fn set(&self, key: &str, value: Value) -> anyhow::Result<()> {
    // Check if this is an entity operation (key contains ".")
    if key.contains(".") {
      let parts: Vec<&str> = key.splitn(2, '.').collect();
      if parts.len() == 2 {
        let entity_name = parts[0];
        let entity_key = parts[1];

        // Attempt to add to entity
        return self.entity_add(entity_name, entity_key, value).await;
      }
    }

    // For regular key-value operation, wrap in a HashMap entity
    if self.is_authenticated() {
      let user_hash = self.get_current_user().unwrap();
      let mut stores = self.auth_stores.write().unwrap();
      let user_store = stores.get_mut(&user_hash).unwrap();

      let mut entities = user_store.entities.lock().unwrap();

      // Create a "default" map if it doesn't exist
      if !entities.contains_key("default") {
        entities.insert(
          "default".to_string(),
          Entities::HashMap(Arc::new(Mutex::new(super::entities::KvHashMap::new())))
        );
      }

      if let Some(Entities::HashMap(map)) = entities.get("default") {
        let mut map = map.lock().unwrap();
        map.insert(key.to_string(), value);
        Ok(())
      } else {
        Err(anyhow::anyhow!("Default map corrupted"))
      }
    } else {
      // Unauthenticated users use the unauth store
      let mut unauth_store = self.unauth_store.write().unwrap();
      let mut entities = unauth_store.entities.lock().unwrap();

      // Create a "default" map if it doesn't exist
      if !entities.contains_key("default") {
        entities.insert(
          "default".to_string(),
          Entities::HashMap(Arc::new(Mutex::new(super::entities::KvHashMap::new())))
        );
      }

      if let Some(Entities::HashMap(map)) = entities.get("default") {
        let mut map = map.lock().unwrap();
        map.insert(key.to_string(), value);
        Ok(())
      } else {
        Err(anyhow::anyhow!("Default map corrupted"))
      }
    }
  }

  async fn get(&self, key: &str) -> Option<Value> {
    // Check if this is an entity operation (key contains ".")
    if key.contains(".") {
      let parts: Vec<&str> = key.splitn(2, '.').collect();
      if parts.len() == 2 {
        let entity_name = parts[0];
        let entity_key = parts[1];

        // Attempt to get from entity
        return match self.entity_get(entity_name, entity_key).await {
          Ok(value) => value,
          Err(_) => None
        };
      }
    }

    // For regular key-value operation, retrieve from default HashMap
    if self.is_authenticated() {
      let user_hash = self.get_current_user().unwrap();
      let stores = self.auth_stores.read().unwrap();

      if let Some(user_store) = stores.get(&user_hash) {
        let entities = user_store.entities.lock().unwrap();

        if let Some(Entities::HashMap(map)) = entities.get("default") {
          let map = map.lock().unwrap();
          return map.get(key).cloned();
        }
      }

      None
    } else {
      // Unauthenticated users can only access unauth store
      let unauth_store = self.unauth_store.read().unwrap();
      let entities = unauth_store.entities.lock().unwrap();

      if let Some(Entities::HashMap(map)) = entities.get("default") {
        let map = map.lock().unwrap();
        return map.get(key).cloned();
      }

      None
    }
  }

  async fn delete(&self, key: &str) -> Option<Value> {
    // Check if this is an entity operation (key contains ".")
    if key.contains(".") {
      let parts: Vec<&str> = key.splitn(2, '.').collect();
      if parts.len() == 2 {
        let entity_name = parts[0];
        let entity_key = parts[1];

        // Attempt to delete from entity
        return match self.entity_delete(entity_name, entity_key).await {
          Ok(value) => value,
          Err(_) => None
        };
      }
    }

    // For regular key-value operation
    if self.is_authenticated() {
      let user_hash = self.get_current_user().unwrap();
      let stores = self.auth_stores.read().unwrap();

      if let Some(user_store) = stores.get(&user_hash) {
        let entities = user_store.entities.lock().unwrap();

        if let Some(Entities::HashMap(map)) = entities.get("default") {
          let mut map = map.lock().unwrap();
          return map.remove(key);
        }
      }

      None
    } else {
      // Unauthenticated users use unauth store
      let unauth_store = self.unauth_store.read().unwrap();
      let entities = unauth_store.entities.lock().unwrap();

      if let Some(Entities::HashMap(map)) = entities.get("default") {
        let mut map = map.lock().unwrap();
        return map.remove(key);
      }

      None
    }
  }

  async fn create_entity(&self, entity_type: &str, name: &str) -> anyhow::Result<()> {
    let entity = match entity_type.to_lowercase().as_str() {
      "set" => Entities::Set(Arc::new(Mutex::new(super::entities::KvSet::new()))),
      "hashmap" => Entities::HashMap(Arc::new(Mutex::new(super::entities::KvHashMap::new()))),
      _ => return Err(anyhow::anyhow!("Unknown entity type: {}", entity_type)),
    };

    if self.is_authenticated() {
      let user_hash = self.get_current_user().unwrap();
      let stores = self.auth_stores.read().unwrap();

      if let Some(user_store) = stores.get(&user_hash) {
        let mut entities = user_store.entities.lock().unwrap();
        entities.insert(name.to_string(), entity);
        Ok(())
      } else {
        Err(anyhow::anyhow!("User store not found"))
      }
    } else {
      // Unauthenticated users use unauth store
      let unauth_store = self.unauth_store.read().unwrap();
      let mut entities = unauth_store.entities.lock().unwrap();
      entities.insert(name.to_string(), entity);
      Ok(())
    }
  }

  async fn entity_add(&self, entity_name: &str, key: &str, value: Value) -> anyhow::Result<()> {
    if self.is_authenticated() {
      let user_hash = self.get_current_user().unwrap();

      // Check if entity exists
      let entity_exists = {
        let stores = self.auth_stores.read().unwrap();
        let user_store = stores.get(&user_hash).ok_or_else(|| anyhow::anyhow!("User store not found"))?;
        let entities = user_store.entities.lock().unwrap();
        entities.contains_key(entity_name)
      };

      // Create entity if it doesn't exist
      if !entity_exists {
        self.create_entity("hashmap", entity_name).await?;
      }

      // Now perform the operation
      let stores = self.auth_stores.read().unwrap();
      let user_store = stores.get(&user_hash).ok_or_else(|| anyhow::anyhow!("User store not found"))?;
      let entities = user_store.entities.lock().unwrap();

      if let Some(entity) = entities.get(entity_name) {
        match entity {
          Entities::Set(set) => {
            let mut set = set.lock().unwrap();
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
      }

      Ok(())
    } else {
      // Unauthenticated users use unauth store

      // Check if entity exists
      let entity_exists = {
        let unauth_store = self.unauth_store.read().unwrap();
        let entities = unauth_store.entities.lock().unwrap();
        entities.contains_key(entity_name)
      };

      // Create entity if it doesn't exist
      if !entity_exists {
        self.create_entity("hashmap", entity_name).await?;
      }

      // Now perform the operation
      let unauth_store = self.unauth_store.read().unwrap();
      let entities = unauth_store.entities.lock().unwrap();

      if let Some(entity) = entities.get(entity_name) {
        match entity {
          Entities::Set(set) => {
            let mut set = set.lock().unwrap();
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
      }

      Ok(())
    }
  }

  async fn entity_get(&self, entity_name: &str, key: &str) -> anyhow::Result<Option<Value>> {
    if self.is_authenticated() {
      let user_hash = self.get_current_user().unwrap();
      let stores = self.auth_stores.read().unwrap();

      if let Some(user_store) = stores.get(&user_hash) {
        let entities = user_store.entities.lock().unwrap();

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
      } else {
        Err(anyhow::anyhow!("User store not found"))
      }
    } else {
      // Unauthenticated users use unauth store
      let unauth_store = self.unauth_store.read().unwrap();
      let entities = unauth_store.entities.lock().unwrap();

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
  }

  async fn entity_delete(&self, entity_name: &str, key: &str) -> anyhow::Result<Option<Value>> {
    if self.is_authenticated() {
      let user_hash = self.get_current_user().unwrap();
      let stores = self.auth_stores.read().unwrap();

      if let Some(user_store) = stores.get(&user_hash) {
        let entities = user_store.entities.lock().unwrap();

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
      } else {
        Err(anyhow::anyhow!("User store not found"))
      }
    } else {
      // Unauthenticated users use unauth store
      let unauth_store = self.unauth_store.read().unwrap();
      let entities = unauth_store.entities.lock().unwrap();

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
}
