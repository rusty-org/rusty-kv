//! GET command implementation.
//!
//! Retrieves stored values by key. Requires authentication.

use anyhow::{Result, anyhow};

use crate::{resp::value::Value, storage::memory::MemoryStore, storage::memory::Store};

/// GET command handler.
///
/// Retrieves a value from the store by its key.
pub struct GetCommand;

impl GetCommand {
  /// Executes the GET command.
  ///
  /// # Arguments
  ///
  /// * `args` - Command arguments (key to retrieve)
  /// * `store` - Memory store to operate on
  ///
  /// # Returns
  ///
  /// * `Ok(Value)` - The retrieved value
  /// * `Err` - Error if key not found or arguments are invalid
  ///
  /// # Example
  ///
  /// ```
  /// // Client sends: GET mykey
  /// let result = GetCommand::execute(vec!["mykey".to_string()], store).await;
  /// ```
  pub async fn execute(args: Vec<String>, store: MemoryStore) -> Result<Value> {
    if !store.is_authenticated() {
      return Err(anyhow!("Authentication required"));
    }

    if args.is_empty() {
      return Err(anyhow!("GET requires a key"));
    }

    let key = &args[0];

    let value = store.get(&key).await;
    if let Some(value) = value {
      Ok(value)
    } else {
      Err(anyhow!("Key {} not found", key))
    }
  }
}
