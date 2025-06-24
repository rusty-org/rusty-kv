//! Data structure implementations.
//!
//! This module provides various data structures that implement the Entity trait,
//! allowing them to be used as storage containers in the key-value server.

use crate::resp::value::Value;

pub mod hmap;
pub mod hset;
pub mod llist;

/// Common interface for data structure entities.
///
/// Defines the standard operations that all entity types must support.
pub trait Entity {
    /// Returns the type name of the entity.
    fn entity_type(&self) -> &'static str;

    /// Gets a value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Some(Value)` - The value if found
    /// * `None` - If the key doesn't exist
    fn get(&self, key: &str) -> Option<Value>;

    /// Sets a key-value pair.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set
    /// * `value` - The value to store
    fn set(&mut self, key: &str, value: Value);

    /// Deletes a key-value pair.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to delete
    ///
    /// # Returns
    ///
    /// * `Some(Value)` - The deleted value if found
    /// * `None` - If the key didn't exist
    fn delete(&mut self, key: &str) -> Option<Value>;
}
