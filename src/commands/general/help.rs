use crate::resp::value::Value;
use anyhow::Result;

#[allow(dead_code)]
pub struct HelpCommand;

impl HelpCommand {
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
