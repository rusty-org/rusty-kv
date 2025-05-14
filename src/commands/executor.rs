use anyhow::{Result, anyhow};
use log::info;

use crate::{
  resp::value::Value,
  storage::{
    db::InternalDB,
    memory::MemoryStore,
    memory::Store
  }
};

use super::{
  acl::auth::AuthCommand,
  general::{
    delete::DeleteCommand, echo::EchoCommand, get::GetCommand, help::HelpCommand,
    ping::PingCommand, set::SetCommand,
  },
};

pub struct CommandExecutor {
  store: MemoryStore,
  db: InternalDB,
}

impl CommandExecutor {
  pub fn new(store: MemoryStore, db: InternalDB) -> Self {
    Self { store, db }
  }

  pub async fn execute(&self, command: &str, args: Vec<String>) -> Result<Value> {
    // Log command with auth status
    let auth_status = if self.store.is_authenticated() { "authenticated" } else { "unauthenticated" };
    info!("Executing command '{}' ({} mode) with args: {:?}", command, auth_status, args);

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
      "AUTH" => AuthCommand::execute(args, self.store.to_owned(), self.db.clone()).await,

      // New entity type creation command
      "CREATE" => {
        if args.len() < 2 {
          return Err(anyhow!("CREATE requires entity type and name"));
        }
        let entity_type = &args[0]; // "hashmap", "set", etc.
        let entity_name = &args[1];

        // Create the entity
        self.store.create_entity(entity_type, entity_name).await?;
        Ok(Value::SimpleString("OK".to_string()))
      },

      _ => Err(anyhow!("Unknown command: {}", command)),
    }
  }
}
