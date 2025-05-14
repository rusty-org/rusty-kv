//! HashSet data structure implementation.
//!
//! Provides a set data structure that implements the Entity trait.

use super::Entity;
use crate::resp::value::Value;
use std::collections::HashSet;

/// A set data structure implementing the Entity trait.
///
/// Stores unique string values and provides set operations.
pub struct KvSet {
  /// The internal HashSet containing the data
  data: HashSet<String>,
}

impl KvSet {
  /// Creates a new empty set.
  pub fn new() -> Self {
    Self {
      data: HashSet::new(),
    }
  }
}

impl Entity for KvSet {
  /// Returns the type name of this entity.
  fn entity_type(&self) -> &'static str {
    "set"
  }

  /// Checks if a key exists in the set.
  ///
  /// # Arguments
  ///
  /// * `key` - The key to check
  ///
  /// # Returns
  ///
  /// * `Some(Value)` - The key as a SimpleString if found
  /// * `None` - If the key doesn't exist
  fn get(&self, key: &str) -> Option<Value> {
    if self.data.contains(key) {
      Some(Value::SimpleString(key.to_string()))
    } else {
      None
    }
  }

  /// Adds a value to the set.
  ///
  /// If the value is a SimpleString, that string is added.
  /// Otherwise, the key is added.
  ///
  /// # Arguments
  ///
  /// * `_key` - Ignored for sets (used as fallback)
  /// * `value` - Value to add (used if SimpleString)
  fn set(&mut self, _key: &str, value: Value) {
    if let Value::SimpleString(val) = value {
      self.data.insert(val);
    }
  }

  /// Removes a key from the set.
  ///
  /// # Arguments
  ///
  /// * `key` - The key to remove
  ///
  /// # Returns
  ///
  /// * `Some(Value)` - The removed key as a SimpleString
  /// * `None` - If the key wasn't in the set
  fn delete(&mut self, key: &str) -> Option<Value> {
    if self.data.remove(key) {
      Some(Value::SimpleString(key.to_string()))
    } else {
      None
    }
  }
}
