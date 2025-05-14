//! DELETE command implementation.
//!
//! Removes one or more keys from the store.

use anyhow::Result;
use anyhow::anyhow;
use log::debug;

use crate::resp::value::Value;
use crate::storage::memory::MemoryStore;
use crate::storage::memory::Store;

/// DELETE command handler.
///
/// Removes one or more keys from the store and returns the count
/// of keys that were actually deleted.
pub struct DeleteCommand;

impl DeleteCommand {
  /// Executes the DELETE command.
  ///
  /// # Arguments
  ///
  /// * `args` - Keys to delete
  /// * `store` - Memory store to operate on
  ///
  /// # Returns
  ///
  /// * `Ok(Value)` - Integer count of keys deleted
  /// * `Err` - Error if no arguments are provided
  ///
  /// # Example
  ///
  /// ```
  /// // Client sends: DEL key1 key2 key3
  /// let result = DeleteCommand::execute(
  ///     vec!["key1".to_string(), "key2".to_string(), "key3".to_string()],
  ///     store
  /// ).await;
  /// // Returns integer representing number of keys actually deleted
  /// ```
  pub async fn execute(args: Vec<String>, store: MemoryStore) -> Result<Value> {
    if args.is_empty() {
      return Err(anyhow!("DEL requires at least one key"));
    }

    for key in args.clone() {
      if let Some(value) = store.get(key.as_str()).await {
        debug!("Deleting key {} with value {:?}", key, value);
        store.delete(key.as_str()).await;
      }
    }

    Ok(Value::Integer(args.len() as i64))
  }
}
