use crate::resp::value::Value;
use anyhow::{Result, anyhow};

pub struct EchoCommand;

impl EchoCommand {
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
