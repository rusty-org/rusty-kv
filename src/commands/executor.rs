use anyhow::{Result, anyhow};

use crate::{resp::value::Value, storage::memory::MemoryStore};

use super::{
  acl::auth::AuthCommand,
  general::{
    delete::DeleteCommand, echo::EchoCommand, get::GetCommand, help::HelpCommand,
    ping::PingCommand, set::SetCommand,
  },
};

pub struct CommandExecutor {
  store: MemoryStore,
}

impl CommandExecutor {
  pub fn new(store: MemoryStore) -> Self {
    Self { store }
  }

  pub async fn execute(&self, command: &str, args: Vec<String>) -> Result<Value> {
    match command {
      // @INFO Utility commands
      "PING" => PingCommand::execute(args),
      "HELP" => HelpCommand::execute(args),
      "ECHO" => EchoCommand::execute(args),

      // @INFO Basic commands for data manipulation
      "GET" => GetCommand::execute(args, self.store.to_owned()).await,
      "SET" => SetCommand::execute(args, self.store.to_owned()).await,
      "DEL" => DeleteCommand::execute(args, self.store.to_owned()).await,

      // @INFO ACL commands
      "AUTH" => AuthCommand::execute(args).await,
      _ => Err(anyhow!("Unknown command: {}", command)),
    }
  }
}
