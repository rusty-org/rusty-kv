use crate::resp::value::Value;
use crate::storage::memory::MemoryStore;
use anyhow::{Result, anyhow};

use super::delete::DeleteCommand;
use super::echo::EchoCommand;
use super::get::GetCommand;
use super::help::HelpCommand;
use super::ping::PingCommand;
use super::set::SetCommand;

pub struct CommandExecutor;

impl CommandExecutor {
  pub fn new() -> Self {
    Self
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
      "DEL" => self.del(args, store).await,
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

  async fn del(&self, args: Vec<String>, store: MemoryStore) -> Result<Value> {
    if args.is_empty() {
      return Err(anyhow!("DEL requires at least one key"));
    }

    let del = DeleteCommand::new(store);
    del.execute(args).await
  }
}
