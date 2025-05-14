//! PING command implementation.
//!
//! Simple connectivity check command that returns PONG or echoes
//! an optional message.

use crate::resp::value::Value;
use anyhow::Result;

/// PING command handler.
///
/// Used to test server connectivity and responsiveness.
pub struct PingCommand;

impl PingCommand {
  /// Executes the PING command.
  ///
  /// Returns "PONG" if no arguments are provided, or echoes back
  /// the first argument provided.
  ///
  /// # Arguments
  ///
  /// * `args` - Optional message to echo back
  ///
  /// # Returns
  ///
  /// * `Ok(Value)` - "PONG" or the echoed message
  ///
  /// # Example
  ///
  /// ```
  /// // Client sends: PING
  /// let result = PingCommand::execute(vec![]);
  /// assert_eq!(result.unwrap(), Value::SimpleString("PONG".to_string()));
  ///
  /// // Client sends: PING hello
  /// let result = PingCommand::execute(vec!["hello".to_string()]);
  /// assert_eq!(result.unwrap(), Value::BulkString("hello".to_string()));
  /// ```
  pub fn execute(args: Vec<String>) -> Result<Value> {
    if args.is_empty() {
      Ok(Value::SimpleString("PONG".to_string()))
    } else {
      Ok(Value::BulkString(args[0].clone()))
    }
  }
}
