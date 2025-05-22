//! Entity types for the storage system.
//!
//! Defines the various data structures that can be used for storing data
//! in the key-value server.

use std::collections::{HashMap, HashSet, LinkedList};
use std::sync::{Arc, Mutex};

use crate::resp::value::Value;

/// A set of unique string values.
pub type KvSet = HashSet<String>;

/// A map of string keys to RESP values.
pub type KvHashMap = HashMap<String, Value>;

/// A linked list of string values.
pub type KvLinkedList = LinkedList<String>;

/// Enum representing different types of data structures for storage.
#[derive(Debug)]
pub enum Entities {
  /// A set of unique string values.
  Set(Arc<Mutex<KvSet>>),

  /// A map of string keys to RESP values.
  HashMap(Arc<Mutex<KvHashMap>>),

  /// A linked list of string values.
  LinkedList(Arc<Mutex<KvLinkedList>>),

  /// A hash set (placeholder for future implementation).
  HashSet,

  /// A list (placeholder for future implementation).
  List,

  /// A queue (placeholder for future implementation).
  Queue,
}
