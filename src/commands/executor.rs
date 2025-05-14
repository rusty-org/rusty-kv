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

      // Entity commands (these would need to be implemented)
      "HSET" => {
        // Example of entity handling
        if args.len() < 3 {
          return Err(anyhow!("HSET requires entity name, key and value"));
        }
        let entity_name = &args[0];
        let key = &args[1];
        let value = Value::SimpleString(args[2].clone());

       // Ensure the user is authenticated before allowing HSET
        if !self.store.is_authenticated() {
          return Err(anyhow!("Authentication required for HSET command"));
        }

        // Try to add to existing entity
        match self.store.entity_add(entity_name, key, value.clone()).await {
          Ok(_) => Ok(Value::SimpleString("OK".to_string())),
          // If entity doesn't exist, create it first
          Err(_) => {
            self.store.create_entity("hashmap", entity_name).await?;
            self.store.entity_add(entity_name, key, value).await?;
            Ok(Value::SimpleString("OK".to_string()))
          }
        }
      },

      _ => Err(anyhow!("Unknown command: {}", command)),
    }
  }
}
