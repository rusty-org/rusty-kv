//! HELP command implementation.
//!
//! Provides help text describing available commands.

use crate::resp::value::Value;
use anyhow::Result;

/// HELP command handler.
///
/// Returns help text with a list of available commands and brief descriptions.
#[allow(dead_code)]
pub struct HelpCommand;

impl HelpCommand {
  /// Executes the HELP command.
  ///
  /// # Arguments
  ///
  /// * `_args` - Ignored arguments
  ///
  /// # Returns
  ///
  /// * `Ok(Value)` - Help text as a bulk string
  ///
  /// # Example
  ///
  /// ```
  /// // Client sends: HELP
  /// let result = HelpCommand::execute(vec![]);
  /// // Returns a bulk string with help text
  /// ```
  pub fn execute(_args: Vec<String>) -> Result<Value> {
    let help_text = "Available commands:\n\
                         PING - Test connection\n\
                         ECHO <message> - Echo back a message\n\
                         GET <key> - Get value for key\n\
                         SET <key> <value> - Set key to value\n\
                         DEL <key> [<key> ...] - Delete keys\n\
                         HELP - Show this help";

    Ok(Value::BulkString(help_text.to_string()))
  }
}
