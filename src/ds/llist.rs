//! Linked List data structure implementation.
//!
//! Provides a linked list data structure that implements the Entity trait.

use super::Entity;
use crate::resp::value::Value;
use std::collections::LinkedList;

/// A linked list data structure implementing the Entity trait.
pub struct KvLinkedList {
    /// The internal LinkedList containing the data
    data: LinkedList<String>,
}

impl KvLinkedList {
    /// Creates a new empty linked list.
    pub fn new() -> Self {
        Self {
            data: LinkedList::new(),
        }
    }

    /// Adds a value to the front of the list.
    pub fn push_front(&mut self, value: String) {
        self.data.push_front(value);
    }

    /// Adds a value to the back of the list.
    pub fn push_back(&mut self, value: String) {
        self.data.push_back(value);
    }

    /// Removes and returns the first value from the list.
    pub fn pop_front(&mut self) -> Option<String> {
        self.data.pop_front()
    }

    /// Removes and returns the last value from the list.
    pub fn pop_back(&mut self) -> Option<String> {
        self.data.pop_back()
    }
}

impl Entity for KvLinkedList {
    /// Returns the type name of this entity.
    fn entity_type(&self) -> &'static str {
        "linkedlist"
    }

    /// Gets a value from the list by index (stored as a string key).
    ///
    /// # Arguments
    ///
    /// * `key` - The index as a string to look up
    ///
    /// # Returns
    ///
    /// * `Some(Value)` - The value if found
    /// * `None` - If the index doesn't exist
    fn get(&self, key: &str) -> Option<Value> {
        if let Ok(index) = key.parse::<usize>() {
            self.data
                .iter()
                .nth(index)
                .map(|val| Value::SimpleString(val.clone()))
        } else {
            None
        }
    }

    /// Adds a value to the back of the list.
    ///
    /// # Arguments
    ///
    /// * `_key` - Ignored for linked lists
    /// * `value` - Value to add
    fn set(&mut self, _key: &str, value: Value) {
        if let Value::SimpleString(val) = value {
            self.push_back(val);
        }
    }

    /// Removes a value by index.
    ///
    /// # Arguments
    ///
    /// * `key` - The index as a string to remove
    ///
    /// # Returns
    ///
    /// * `Some(Value)` - The removed value
    /// * `None` - If the index doesn't exist
    fn delete(&mut self, key: &str) -> Option<Value> {
        if let Ok(index) = key.parse::<usize>() {
            let mut temp_list = LinkedList::new();
            let mut removed = None;
            let mut current_index = 0;

            // Move elements to a temporary list, skipping the one to delete
            while let Some(item) = self.data.pop_front() {
                if current_index == index {
                    removed = Some(Value::SimpleString(item));
                } else {
                    temp_list.push_back(item);
                }
                current_index += 1;
            }

            // Restore the list without the deleted element
            self.data = temp_list;
            removed
        } else {
            None
        }
    }
}
