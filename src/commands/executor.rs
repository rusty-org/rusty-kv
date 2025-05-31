//! Command execution router and dispatcher.
//!
//! This module handles the parsing, routing, and execution of all commands.
//! It maps command strings to their corresponding handler implementations.

use anyhow::{Result, anyhow};
use log::info;

use crate::{
  commands::acl::whoami::WhoAmi,
  resp::value::Value,
  storage::{
    db::InternalDB,
    memory::{MemoryStore, Store},
  },
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
  pub async fn execute(&self, command: &str, args: Vec<Value>) -> Result<Value> {
    // Log command with auth status
    let auth_status = if self.store.is_authenticated() {
      "authenticated"
    } else {
      "unauthenticated"
    };
    info!(
      "Executing command '{}' ({} mode) with args: {:?}",
      command, auth_status, args
    );

    // Convert Values to strings for commands that still expect strings
    let string_args: Vec<String> = args
      .iter()
      .map(|v| match v {
        Value::SimpleString(s) => s.clone(),
        Value::BulkString(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Boolean(b) => b.to_string(),
        _ => "".to_string(),
      })
      .collect();

    match command {
      // @INFO Utility commands
      "PING" => PingCommand::execute(string_args),
      "HELP" => HelpCommand::execute(string_args),
      "ECHO" => EchoCommand::execute(string_args),

      // @INFO Basic commands for data manipulation
      "GET" => GetCommand::execute(string_args, self.store.to_owned()).await,
      "SET" => SetCommand::execute(string_args, self.store.to_owned(), args).await,
      "DEL" => DeleteCommand::execute(string_args, self.store.to_owned()).await,

      // @INFO ACL commands
      "AUTH" => AuthCommand::execute(string_args, self.store.to_owned(), self.db.clone()).await,
      "WHOAMI" => WhoAmi::execute(self.store.clone(), self.db.clone()).await,

      // @INFO Catch-all for unknown commands
      _ => Err(anyhow!("Unknown command: {}", command)),
    }
  }
}
