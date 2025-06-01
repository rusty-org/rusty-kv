//! Entity types for the storage system.
//!
//! Defines the various data structures that can be used for storing data
//! in the key-value server.

use std::collections::{HashMap, HashSet, LinkedList};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::commands::general::set::Options;
use crate::resp::value::Value;

/// @NOTE Helper type aliases
/// -------------------------------------------------------------------

/// Helper type for storing key-value pairs with optional modifiers.
pub type KvMapArgs = HashMap<Options, u64>;
/// Represents a the Value as the first element and arguments map as the last element
/// and the SystemTime as the second element to store the time of insertion.
pub type KvMapPair = (Value, SystemTime, KvMapArgs);

/// -------------------------------------------------------------------

/// A set of unique string values.
pub type KvSet = HashSet<String>;

/// A map of string keys to RESP values.
pub type KvHashMap = HashMap<String, KvMapPair>;

/// A linked list of string values.
pub type KvLinkedList = LinkedList<String>;

/// Enum representing different types of data structures for storage.
#[derive(Debug)]
pub enum Entities {
  /// A set of unique string values.
  _Set(Arc<Mutex<KvSet>>),

  /// A map of string keys to RESP values.
  HashMap(Arc<Mutex<KvHashMap>>),

  /// A linked list of string values.
  _LinkedList(Arc<Mutex<KvLinkedList>>),

  /// A hash set (placeholder for future implementation).
  _HashSet,

  /// A list (placeholder for future implementation).
  _List,

  /// A queue (placeholder for future implementation).
  _Queue,
}
