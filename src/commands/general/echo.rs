//! ECHO command implementation.
//!
//! Echoes back the message provided as an argument.

use crate::resp::value::Value;
use anyhow::{Result, anyhow};

/// ECHO command handler.
///
/// Returns the provided message back to the client.
pub struct EchoCommand;

impl EchoCommand {
  /// Executes the ECHO command.
  ///
  /// # Arguments
  ///
  /// * `args` - Message to echo back
  ///
  /// # Returns
  ///
  /// * `Ok(Value)` - The echoed message
  /// * `Err` - Error if no arguments are provided
  ///
  /// # Example
  ///
  /// ```
  /// // Client sends: ECHO hello world
  /// let result = EchoCommand::execute(vec!["hello".to_string(), "world".to_string()]);
  /// // Returns "hello" as a bulk string (behavior currently only echoes first arg)
  /// ```
  pub fn execute(args: Vec<String>) -> Result<Value> {
    let message = args.clone().join(" ");

    if !args.is_empty() {
      Ok(Value::BulkString(args[0].clone()))
    } else if !message.is_empty() {
      Ok(Value::BulkString(message.clone()))
    } else {
      Err(anyhow!("ECHO requires at least one argument"))
    }
  }
}
