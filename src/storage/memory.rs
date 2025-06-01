//! In-memory storage implementation for the key-value server.
//!
//! Provides a thread-safe, multi-user in-memory data store with support
//! for different entity types (HashMaps, Sets) and authentication.

use std::{
  collections::HashMap,
  sync::{Arc, Mutex, RwLock},
  time::SystemTime,
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
    options: HashMap<Options, u64>,
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
  async fn set(&self, key: &str, value: Value, args: HashMap<Options, u64>) -> anyhow::Result<()> {
    if !self.is_authenticated() {
      return Err(anyhow::anyhow!("Authentication required"));
    }

    debug!("Got extra options: {:?}", args);

    // @TODO: handle where user would want to divider their data into different entities like this
    // @TODO: `SET admin.foo bar` would set a value in the "admin" entity with key "foo"
    // // Check if this is an entity operation (key contains ".")
    // if key.contains(".") {
    //   let parts: Vec<&str> = key.splitn(2, '.').collect();
    //   if parts.len() == 2 {
    //     let entity_name = parts[0];
    //     let entity_key = parts[1];

    //     // Attempt to add to entity
    //     return self.entity_add(entity_name, entity_key, value).await;
    //   }
    // }

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
      map.insert(key.to_string(), (value, SystemTime::now(), args));
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

    // @TODO: handle where user would want to divider their data into different entities like this
    // @TODO: `GET admin.foo` would get a value in the "admin" entity with key "foo"
    // // Check if this is an entity operation (key contains ".")
    // if key.contains(".") {
    //   let parts: Vec<&str> = key.splitn(2, '.').collect();
    //   if parts.len() == 2 {
    //     let entity_name = parts[0];
    //     let entity_key = parts[1];

    //     // Attempt to get from entity
    //     return match self.entity_get(entity_name, entity_key).await {
    //       Ok(value) => value,
    //       Err(_) => None,
    //     };
    //   }
    // }

    // For regular key-value operation, retrieve from default HashMap
    let user_hash = self.get_current_user().unwrap();
    let stores = self.auth_stores.read().unwrap();

    if let Some(user_store) = stores.get(&user_hash) {
      let entities = user_store.entities.lock().unwrap();

      if let Some(Entities::HashMap(map)) = entities.get("default") {
        // Get the map and check for the key
        let map = map.lock().unwrap();
        // Get the value tuple for the key
        let val_tuple = map.get(key);

        if let Some((value, _time, args)) = val_tuple {
          // Check for expiration if Ex option is set (in seconds)
          if let Some(&expiry_ms) = args.get(&Options::Ex) {
            let elapsed = SystemTime::elapsed(_time).unwrap();
            if elapsed.as_secs() >= expiry_ms as u64 {
              debug!("Key '{}' has expired", key);
              return None; // Key has expired
            }
          }

          // Check for expiration if Px option is set (in milliseconds)
          if let Some(&expiry_ms) = args.get(&Options::Px) {
            let elapsed = SystemTime::elapsed(_time).unwrap();
            if elapsed.as_millis() >= expiry_ms as u128 {
              debug!("Key '{}' has expired", key);
              return None; // Key has expired
            }
          }
          return Some(value.clone()); // Return the value if not expired
        };
        debug!("Key '{}' not found in default HashMap", key);
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

    // @TODO: handle where user would want to divider their data into different entities like this
    // @TODO: `DEL admin.foo` would delete a value in the "admin" entity with key "foo"
    // // Check if this is an entity operation (key contains ".")
    // if key.contains(".") {
    //   let parts: Vec<&str> = key.splitn(2, '.').collect();
    //   if parts.len() == 2 {
    //     let entity_name = parts[0];
    //     let entity_key = parts[1];

    //     // Attempt to delete from entity
    //     return match self.entity_delete(entity_name, entity_key).await {
    //       Ok(value) => value,
    //       Err(_) => None,
    //     };
    //   }
    // }

    // For regular key-value operation
    let user_hash = self.get_current_user().unwrap();
    let stores = self.auth_stores.read().unwrap();

    if let Some(user_store) = stores.get(&user_hash) {
      let entities = user_store.entities.lock().unwrap();

      if let Some(Entities::HashMap(map)) = entities.get("default") {
        let mut map = map.lock().unwrap();
        return map.remove(key).map(|(value, _time, _args)| value);
      }
    }

    None
  }
}
