use anyhow::{Result, anyhow};

use crate::{resp::value::Value, storage::memory::MemoryStore, storage::memory::Store};

pub struct GetCommand {
  store: MemoryStore,
}

impl GetCommand {
  pub fn new(store: MemoryStore) -> Self {
    Self { store }
  }

  pub async fn execute(&self, args: Vec<String>) -> Result<Value> {
    if args.is_empty() {
      return Err(anyhow!("GET requires a key"));
    }

    let key = &args[0];

    let value = self.store.get(&key).await;
    if let Some(value) = value {
      Ok(value)
    } else {
      Err(anyhow!("Key {} not found", key))
    }
  }
}
