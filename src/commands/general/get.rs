use anyhow::{Result, anyhow};

use crate::{resp::value::Value, storage::memory::MemoryStore, storage::memory::Store};

pub struct GetCommand;

impl GetCommand {
  pub async fn execute(args: Vec<String>, store: MemoryStore) -> Result<Value> {
    if args.is_empty() {
      return Err(anyhow!("GET requires a key"));
    }

    let key = &args[0];

    let value = store.get(&key).await;
    if let Some(value) = value {
      Ok(value)
    } else {
      Err(anyhow!("Key {} not found", key))
    }
  }
}
