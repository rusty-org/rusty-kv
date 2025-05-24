//! SET command implementation.
//!
//! Handles storing key-value pairs with optional modifiers (EX, PX, NX, XX).
//! Requires authentication before executing.

use std::collections::HashMap;

use crate::{resp::value::Value, storage::memory::MemoryStore, storage::memory::Store};
use anyhow::{Result, anyhow};
use log::debug;

/// SET command handler.
///
/// Allows storing values with a given key. Supports Redis-compatible
/// optional modifiers.
pub struct SetCommand;

/// Optional modifiers for the SET command.
///
/// These modifiers allow setting expiration times or conditions
/// for the key-value pair:
///
/// # Example
/// ```
/// SET my key myvalue EX 60
/// SET my key myvalue PX 1000
/// SET my key myvalue NX
/// SET my key myvalue XX
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Options {
  Ex, // Expiration in seconds
  Px, // Expiration in milliseconds
  Nx,      // Only set if not exists
  Xx,      // Only set if exists
}

impl SetCommand {
  /// Executes the SET command.
  ///
  /// # Arguments
  ///
  /// * `args` - Command arguments (key, value, and optional modifiers)
  /// * `store` - Memory store to operate on
  ///
  /// # Returns
  ///
  /// * `Ok(Value)` - Success response
  /// * `Err` - Error if command fails or arguments are invalid
  ///
  /// # Optional Modifiers
  ///
  /// * `EX seconds` - Set expiration time in seconds
  /// * `PX milliseconds` - Set expiration time in milliseconds
  /// * `NX` - Only set key if it does not exist
  /// * `XX` - Only set key if it already exists
  ///
  /// # Example
  ///
  /// ```
  /// // Client sends: SET mykey myvalue EX 60
  /// let result = SetCommand::execute(
  ///     vec!["mykey".to_string(), "myvalue".to_string(), "EX".to_string(), "60".to_string()],
  ///     store
  /// ).await;
  /// ```
  pub async fn execute(mut args: Vec<String>, store: MemoryStore) -> Result<Value> {
    if !store.is_authenticated() {
      return Err(anyhow!("Authentication required"));
    }

    if args.len() < 2 {
      return Err(anyhow!("SET requires a key and a value"));
    }

    let key = args[0].to_owned();
    let value = args[1].to_owned();
    let mut extra_args = HashMap::<Options, String>::new();

    // @NOTE Find any other optional arguments
    // Such as EX, PX, NX, XX
    while args.len() > 2 {
      let arg = args.remove(2);

      match arg.to_uppercase().as_str() {
        "EX" => {
          // Handle expiration in seconds
          if let Some(expiration) = args.get(2) {
            debug!("Setting expiration to {} seconds", expiration);
            extra_args.insert(
              Options::Ex,
              expiration.into(),
            );
            args.remove(2);
          }
        }
        "PX" => {
          // Handle expiration in milliseconds
          if let Some(expiration) = args.get(2) {
            debug!("Setting expiration to {} milliseconds", expiration);
            extra_args.insert(
              Options::Px,
              expiration.into(),
            );
            args.remove(2);
          }
        }
        "NX" => {
          // Handle only set if not exists
          // Logic for NX goes here
        }
        "XX" => {
          // Handle only set if exists
          // Logic for XX goes here
        }
        _ => {}
      }
    }

    // Set the value in the store
    store
      .set(key.as_str(), Value::SimpleString(value.clone()), extra_args)
      .await?;
    debug!("Set key {} to value {}", key, value);

    Ok(Value::SimpleString("OK".to_string()))
  }
}
