//! Command execution router and dispatcher.
//!
//! This module handles the parsing, routing, and execution of all commands.
//! It maps command strings to their corresponding handler implementations.

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

/// Command executor and router.
///
/// Routes incoming commands to the appropriate command handler
/// and manages shared state (storage, database connections).
pub struct CommandExecutor {
  /// Shared memory store for key-value operations
  store: MemoryStore,
  /// Database connection for persistent storage
  db: InternalDB,
}

impl CommandExecutor {
  /// Creates a new command executor.
  ///
  /// # Arguments
  ///
  /// * `store` - Shared memory store
  /// * `db` - Database connection
  ///
  /// # Returns
  ///
  /// A new CommandExecutor instance
  pub fn new(store: MemoryStore, db: InternalDB) -> Self {
    Self { store, db }
  }

  /// Executes a command with its arguments.
  ///
  /// Routes the command to the appropriate handler based on the command name.
  ///
  /// # Arguments
  ///
  /// * `command` - Command name (e.g., "GET", "SET", "PING")
  /// * `args` - Command arguments
  ///
  /// # Returns
  ///
  /// * `Ok(Value)` - Command execution result
  /// * `Err` - Error if command is invalid or execution fails
  ///
  /// # Example
  ///
  /// ```
  /// // Execute a GET command
  /// let result = executor.execute("GET", vec!["mykey".to_string()]).await;
  /// ```
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
