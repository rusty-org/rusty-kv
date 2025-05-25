//! In-memory storage implementation for the key-value server.
//!
//! Provides a thread-safe, multi-user in-memory data store with support
//! for different entity types (HashMaps, Sets) and authentication.

use std::{
  collections::{HashMap, LinkedList},
  sync::{Arc, Mutex, RwLock},
};

use log::{debug, info};

use super::entities::{Entities, KvHashMap};
use crate::{commands::general::set::Options, resp::value::Value};

/// Main in-memory storage structure.
///
/// Provides authenticated access to user-specific data stores.
#[derive(Clone)]
pub struct MemoryStore {
  /// Store for authenticated users, keyed by user credential hash
  auth_stores: Arc<RwLock<HashMap<String, UserStore>>>,
  /// Current user's credential hash (if authenticated)
  current_user: Arc<RwLock<Option<String>>>,
}

/// Represents a single user's data store.
///
/// Contains all entities (HashMaps, Sets, etc.) owned by a specific user.
#[derive(Clone, Debug)]
pub struct UserStore {
  /// Stores entity references for various data types
  /// Key is entity name, value is the entity (HashMap, Set, etc)
  entities: Arc<Mutex<HashMap<String, Entities>>>,
}

impl UserStore {
  /// Creates a new empty UserStore.
  fn new() -> Self {
    Self {
      entities: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

/// Interface for storage operations.
///
/// Defines the standard operations that all storage implementations must provide.
pub trait Store {
  /// Creates a new store instance.
  fn new() -> Self;

  /// Sets a key-value pair in the store.
  ///
  /// # Arguments
  ///
  /// * `key` - The key to set
  /// * `value` - The value to store
  async fn set(
    &self,
    key: &str,
    value: Value,
    options: HashMap<Options, u128>,
  ) -> anyhow::Result<()>;

  /// Gets a value from the store by key.
  ///
  /// # Arguments
  ///
  /// * `key` - The key to look up
  ///
  /// # Returns
  ///
  /// * `Some(Value)` - The value if found
  /// * `None` - If the key doesn't exist
  async fn get(&self, key: &str) -> Option<Value>;

  /// Deletes a key-value pair from the store.
  ///
  /// # Arguments
  ///
  /// * `key` - The key to delete
  ///
  /// # Returns
  ///
  /// * `Some(Value)` - The deleted value if found
  /// * `None` - If the key didn't exist
  async fn delete(&self, key: &str) -> Option<Value>;

  /// Sets the current authenticated user.
  ///
  /// # Arguments
  ///
  /// * `user_hash` - Credential hash for the authenticated user, or None to clear
  fn set_current_user(&self, user_hash: Option<String>);

  /// Gets the current authenticated user's credential hash.
  ///
  /// # Returns
  ///
  /// * `Some(String)` - Credential hash if a user is authenticated
  /// * `None` - If no user is authenticated
  fn get_current_user(&self) -> Option<String>;

  /// Checks if a user is currently authenticated.
  ///
  /// # Returns
  ///
  /// * `true` - A user is authenticated
  /// * `false` - No user is authenticated
  fn is_authenticated(&self) -> bool;

  /// Creates a new entity of the specified type.
  ///
  /// # Arguments
  ///
  /// * `entity_type` - Type of entity to create (e.g., "hashmap", "set")
  /// * `name` - Name to assign to the new entity
  async fn create_entity(&self, entity_type: &str, name: &str) -> anyhow::Result<()>;

  /// Adds a value to an entity.
  ///
  /// # Arguments
  ///
  /// * `entity_name` - Name of the entity to add to
  /// * `key` - Key within the entity
  /// * `value` - Value to add
  async fn entity_add(&self, entity_name: &str, key: &str, value: Value) -> anyhow::Result<()>;

  /// Gets a value from an entity.
  ///
  /// # Arguments
  ///
  /// * `entity_name` - Name of the entity to get from
  /// * `key` - Key within the entity
  ///
  /// # Returns
  ///
  /// * `Ok(Some(Value))` - Value was found
  /// * `Ok(None)` - Key doesn't exist in entity
  /// * `Err(...)` - Entity doesn't exist or other error
  async fn entity_get(&self, entity_name: &str, key: &str) -> anyhow::Result<Option<Value>>;

  /// Deletes a value from an entity.
  ///
  /// # Arguments
  ///
  /// * `entity_name` - Name of the entity to delete from
  /// * `key` - Key within the entity to delete
  ///
  /// # Returns
  ///
  /// * `Ok(Some(Value))` - Value was deleted
  /// * `Ok(None)` - Key didn't exist in entity
  /// * `Err(...)` - Entity doesn't exist or other error
  async fn entity_delete(&self, entity_name: &str, key: &str) -> anyhow::Result<Option<Value>>;
}

impl Store for MemoryStore {
  /// Creates a new empty MemoryStore instance.
  fn new() -> Self {
    info!("Initializing memory store for authenticated users only");
    Self {
      auth_stores: Arc::new(RwLock::new(HashMap::new())),
      current_user: Arc::new(RwLock::new(None)),
    }
  }

  /// Sets the current authenticated user and initializes their store if needed.
  ///
  /// # Arguments
  ///
  /// * `user_hash` - Credential hash for the user, or None to clear authentication
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

  /// Gets the current authenticated user's credential hash.
  fn get_current_user(&self) -> Option<String> {
    self.current_user.read().unwrap().clone()
  }

  /// Checks if a user is currently authenticated.
  fn is_authenticated(&self) -> bool {
    self.current_user.read().unwrap().is_some()
  }

  /// Sets a key-value pair in the store.
  ///
  /// If the key contains a dot, it's treated as an entity operation.
  /// Otherwise, it's stored in the default HashMap.
  async fn set(&self, key: &str, value: Value, args: HashMap<Options, u128>) -> anyhow::Result<()> {
    if !self.is_authenticated() {
      return Err(anyhow::anyhow!("Authentication required"));
    }

    debug!("Got extra options: {:?}", args);

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
    let user_hash = self.get_current_user().unwrap();
    let mut stores = self.auth_stores.write().unwrap();
    let user_store = stores.get_mut(&user_hash).unwrap();

    let mut entities = user_store.entities.lock().unwrap();

    // Create a "default" map if it doesn't exist
    if !entities.contains_key("default") {
      entities.insert(
        "default".to_string(),
        Entities::HashMap(Arc::new(Mutex::new(KvHashMap::new()))),
      );
    }

    // Insert the key-value pair into the default HashMap
    if let Some(Entities::HashMap(map)) = entities.get("default") {
      let mut map = map.lock().unwrap();
      map.insert(key.to_string(), (value, args));
      Ok(())
    } else {
      Err(anyhow::anyhow!("Default map corrupted"))
    }
  }

  /// Gets a value from the store by key.
  ///
  /// If the key contains a dot, it's treated as an entity operation.
  /// Otherwise, it looks in the default HashMap.
  async fn get(&self, key: &str) -> Option<Value> {
    if !self.is_authenticated() {
      return None;
    }

    // Check if this is an entity operation (key contains ".")
    if key.contains(".") {
      let parts: Vec<&str> = key.splitn(2, '.').collect();
      if parts.len() == 2 {
        let entity_name = parts[0];
        let entity_key = parts[1];

        // Attempt to get from entity
        return match self.entity_get(entity_name, entity_key).await {
          Ok(value) => value,
          Err(_) => None,
        };
      }
    }

    // For regular key-value operation, retrieve from default HashMap
    let user_hash = self.get_current_user().unwrap();
    let stores = self.auth_stores.read().unwrap();

    if let Some(user_store) = stores.get(&user_hash) {
      let entities = user_store.entities.lock().unwrap();

      if let Some(Entities::HashMap(map)) = entities.get("default") {
        let map = map.lock().unwrap();
        return map.get(key).map(|(value, _args)| value.clone());
      }
    }

    None
  }

  /// Deletes a key-value pair from the store.
  ///
  /// If the key contains a dot, it's treated as an entity operation.
  /// Otherwise, it removes from the default HashMap.
  async fn delete(&self, key: &str) -> Option<Value> {
    if !self.is_authenticated() {
      return None;
    }

    // Check if this is an entity operation (key contains ".")
    if key.contains(".") {
      let parts: Vec<&str> = key.splitn(2, '.').collect();
      if parts.len() == 2 {
        let entity_name = parts[0];
        let entity_key = parts[1];

        // Attempt to delete from entity
        return match self.entity_delete(entity_name, entity_key).await {
          Ok(value) => value,
          Err(_) => None,
        };
      }
    }

    // For regular key-value operation
    let user_hash = self.get_current_user().unwrap();
    let stores = self.auth_stores.read().unwrap();

    if let Some(user_store) = stores.get(&user_hash) {
      let entities = user_store.entities.lock().unwrap();

      if let Some(Entities::HashMap(map)) = entities.get("default") {
        let mut map = map.lock().unwrap();
        return map.remove(key).map(|(value, _args)| value);
      }
    }

    None
  }

  /// Creates a new entity of the specified type.
  ///
  /// # Arguments
  ///
  /// * `entity_type` - Type of entity to create (e.g., "hashmap", "set")
  /// * `name` - Name to assign to the new entity
  async fn create_entity(&self, entity_type: &str, name: &str) -> anyhow::Result<()> {
    if !self.is_authenticated() {
      return Err(anyhow::anyhow!("Authentication required"));
    }

    // Create the appropriate entity type based on the request
    let entity = match entity_type.to_lowercase().as_str() {
      "set" => Entities::Set(Arc::new(Mutex::new(super::entities::KvSet::new()))),
      "hashmap" => Entities::HashMap(Arc::new(Mutex::new(super::entities::KvHashMap::new()))),
      "linkedlist" => {
        Entities::LinkedList(Arc::new(Mutex::new(super::entities::KvLinkedList::new())))
      }
      _ => return Err(anyhow::anyhow!("Unknown entity type: {}", entity_type)),
    };

    let user_hash = self.get_current_user().unwrap();
    let stores = self.auth_stores.read().unwrap();

    if let Some(user_store) = stores.get(&user_hash) {
      let mut entities = user_store.entities.lock().unwrap();
      entities.insert(name.to_string(), entity);
      Ok(())
    } else {
      Err(anyhow::anyhow!("User store not found"))
    }
  }

  /// Adds a value to an entity.
  ///
  /// Creates the entity if it doesn't exist.
  async fn entity_add(&self, entity_name: &str, key: &str, value: Value) -> anyhow::Result<()> {
    if !self.is_authenticated() {
      return Err(anyhow::anyhow!("Authentication required"));
    }

    let user_hash = self.get_current_user().unwrap();

    // Check if entity exists
    let entity_exists = {
      let stores = self.auth_stores.read().unwrap();
      let user_store = stores
        .get(&user_hash)
        .ok_or_else(|| anyhow::anyhow!("User store not found"))?;
      let entities = user_store.entities.lock().unwrap();
      entities.contains_key(entity_name)
    };

    // Create entity if it doesn't exist
    if !entity_exists {
      self.create_entity("hashmap", entity_name).await?;
    }

    // Now perform the operation
    let stores = self.auth_stores.read().unwrap();
    let user_store = stores
      .get(&user_hash)
      .ok_or_else(|| anyhow::anyhow!("User store not found"))?;
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
        }
        Entities::HashMap(hashmap) => {
          let mut hashmap = hashmap.lock().unwrap();
          hashmap.insert(key.to_string(), (value, HashMap::new()));
        }
        Entities::LinkedList(list) => {
          let mut list = list.lock().unwrap();
          if let Value::SimpleString(val) = &value {
            list.push_back(val.clone());
          } else {
            // Use key as fallback if value isn't a SimpleString
            list.push_back(key.to_string());
          }
        }
        _ => {
          return Err(anyhow::anyhow!(
            "Entity type not supported for this operation"
          ));
        }
      }
    }

    Ok(())
  }

  /// Gets a value from an entity.
  async fn entity_get(&self, entity_name: &str, key: &str) -> anyhow::Result<Option<Value>> {
    if !self.is_authenticated() {
      return Err(anyhow::anyhow!("Authentication required"));
    }

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
          }
          Entities::HashMap(hashmap) => {
            let hashmap = hashmap.lock().unwrap();
            Ok(hashmap.get(key).map(| (value, _args)| value.clone()))
          }
          Entities::LinkedList(list) => {
            let list = list.lock().unwrap();
            // For linked list, we need to iterate to find the key
            for value in list.iter() {
              if value == key {
                return Ok(Some(Value::SimpleString(key.to_string())));
              }
            }
            Ok(None)
          }
          _ => Err(anyhow::anyhow!(
            "Entity type not supported for this operation"
          )),
        }
      } else {
        Err(anyhow::anyhow!("Entity not found: {}", entity_name))
      }
    } else {
      Err(anyhow::anyhow!("User store not found"))
    }
  }

  /// Deletes a value from an entity.
  async fn entity_delete(&self, entity_name: &str, key: &str) -> anyhow::Result<Option<Value>> {
    if !self.is_authenticated() {
      return Err(anyhow::anyhow!("Authentication required"));
    }

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
          }
          Entities::HashMap(hashmap) => {
            let mut hashmap = hashmap.lock().unwrap();
            Ok(hashmap.remove(key).map(|(value, _args)| value))
          }
          Entities::LinkedList(list) => {
            let mut list = list.lock().unwrap();
            // For linked list, we need a stable approach to remove the key
            let mut temp_list = LinkedList::new();
            let mut removed = false;

            // Move all elements except the one we want to remove
            while let Some(value) = list.pop_front() {
              if !removed && value == key {
                removed = true;
                // Skip this item (don't add to temp_list)
              } else {
                temp_list.push_back(value);
              }
            }

            // Replace the original list with our filtered list
            *list = temp_list;

            if removed {
              Ok(Some(Value::SimpleString(key.to_string())))
            } else {
              Ok(None)
            }
          }
          _ => Err(anyhow::anyhow!(
            "Entity type not supported for this operation"
          )),
        }
      } else {
        Err(anyhow::anyhow!("Entity not found: {}", entity_name))
      }
    } else {
      Err(anyhow::anyhow!("User store not found"))
    }
  }
}
