use crate::resp::value::Value;
use anyhow::{Result, anyhow};

#[allow(dead_code)]
pub struct EchoCommand {
  message: String,
}

#[allow(dead_code)]
impl EchoCommand {
  pub fn new(message: String) -> Self {
    Self { message }
  }
}

impl EchoCommand {
  pub fn execute(&self, args: Vec<String>) -> Result<Value> {
    if !args.is_empty() {
      Ok(Value::BulkString(args[0].clone()))
    } else if !self.message.is_empty() {
      Ok(Value::BulkString(self.message.clone()))
    } else {
      Err(anyhow!("ECHO requires at least one argument"))
    }
  }
}
