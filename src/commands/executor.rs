use crate::resp::value::Value;
use crate::storage::memory::MemoryStore;
use anyhow::{Ok, Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::echo::EchoCommand;
use super::get::GetCommand;
use super::help::HelpCommand;
use super::ping::PingCommand;
use super::set::SetCommand;

pub struct CommandExecutor {
  db: Arc<Mutex<HashMap<String, Value>>>,
}

impl CommandExecutor {
  pub fn new() -> Self {
    Self {
      db: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub async fn execute(
    &self,
    command: &str,
    args: Vec<String>,
    store: MemoryStore,
  ) -> Result<Value> {
    match command {
      "PING" => self.ping(args).await,
      "HELP" => self.help(args).await,
      "ECHO" => self.echo(args).await,
      "GET" => self.get(args, store).await,
      "SET" => self.set(args, store).await,
      "DEL" => self.del(args).await,
      _ => Err(anyhow!("Unknown command: {}", command)),
    }
  }

  async fn ping(&self, args: Vec<String>) -> Result<Value> {
    let ping = PingCommand::new();
    ping.execute(args)
  }

  async fn help(&self, args: Vec<String>) -> Result<Value> {
    let help = HelpCommand::new();
    help.execute(args)
  }

  async fn echo(&self, args: Vec<String>) -> Result<Value> {
    let echo = EchoCommand::new(args.join(" "));
    echo.execute(args)
  }

  async fn get(&self, args: Vec<String>, store: MemoryStore) -> Result<Value> {
    if args.is_empty() {
      return Err(anyhow!("GET requires a key"));
    }

    let get = GetCommand::new(store);
    get.execute(args).await
  }

  async fn set(&self, args: Vec<String>, store: MemoryStore) -> Result<Value> {
    if args.len() < 2 {
      return Err(anyhow!("SET requires a key and a value"));
    }

    let set = SetCommand::new(store);
    set.execute(args).await
  }

  async fn del(&self, args: Vec<String>) -> Result<Value> {
    if args.is_empty() {
      return Err(anyhow!("DEL requires at least one key"));
    }

    let mut count = 0;
    let mut db = self.db.lock().await;

    for key in args {
      if db.remove(&key).is_some() {
        count += 1;
      }
    }

    Ok(Value::Integer(count))
  }
}
