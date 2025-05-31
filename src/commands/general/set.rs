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
  Nx, // Only set if not exists
  Xx, // Only set if exists
}

impl SetCommand {
  /// Executes the SET command.
  ///
  /// # Arguments
  ///
  /// * `args` - Command arguments (key, value, and optional modifiers)
  /// * `store` - Memory store to operate on
  /// * `orig_args` - Original value objects to preserve type
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
  ///     store,
  ///     orig_args
  /// ).await;
  /// ```
  pub async fn execute(
    mut args: Vec<String>,
    store: MemoryStore,
    orig_args: Vec<Value>,
  ) -> Result<Value> {
    if !store.is_authenticated() {
      return Err(anyhow!("Authentication required"));
    }

    if args.len() < 2 {
      return Err(anyhow!("SET requires a key and a value"));
    }

    let key = args[0].to_owned();
    let mut extra_args = HashMap::<Options, u64>::new();

    // Get the original value with its type preserved
    let value = if orig_args.len() > 1 {
      orig_args[1].clone()
    } else {
      Value::SimpleString(args[1].clone())
    };

    // @NOTE Find any other optional arguments
    // Such as EX, PX, NX, XX
    let mut arg_index = 2;
    while arg_index < args.len() {
      let arg = args[arg_index].clone();
      arg_index += 1;

      match arg.to_uppercase().as_str() {
        "EX" => {
          // Handle expiration in seconds
          if let Some(expiration) = args.get(arg_index) {
            debug!("Setting expiration to {} seconds", expiration);

            // Parse the expiration value and add that to the extra_args
            match expiration.parse::<u64>() {
              Ok(exp) => {
                extra_args.insert(Options::Ex, exp);
              }
              Err(_) => {
                return Err(anyhow!("Invalid expiration value: {}", expiration));
              }
            }

            // Move to the next argument
            arg_index += 1;
          }
        }
        "PX" => {
          // Handle expiration in milliseconds
          if let Some(expiration) = args.get(arg_index) {
            debug!("Setting expiration to {} milliseconds", expiration);

            // Parse the expiration value and add that to the extra_args
            match expiration.parse::<u64>() {
              Ok(exp) => {
                extra_args.insert(Options::Px, exp);
              }
              Err(_) => {
                return Err(anyhow!("Invalid expiration value: {}", expiration));
              }
            }

            // Move to the next argument
            arg_index += 1;
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
    store.set(key.as_str(), value.clone(), extra_args).await?;

    // Log with the display representation of the value
    let display_value = match &value {
      Value::SimpleString(s) => s.clone(),
      Value::BulkString(s) => s.clone(),
      Value::Integer(i) => i.to_string(),
      Value::Boolean(b) => b.to_string(),
      _ => format!("{:?}", value),
    };
    debug!("Set key {} to value {}", key, display_value);

    Ok(Value::SimpleString("OK".to_string()))
  }
}
