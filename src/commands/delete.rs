use anyhow::Result;
use anyhow::anyhow;
use log::debug;

use crate::resp::value::Value;
use crate::storage::memory::MemoryStore;
use crate::storage::memory::Store;

pub struct DeleteCommand {
  store: MemoryStore,
}

impl DeleteCommand {
  pub fn new(store: MemoryStore) -> Self {
    Self { store }
  }

  pub async fn execute(&self, args: Vec<String>) -> Result<Value> {
    if args.is_empty() {
      return Err(anyhow!("DEL requires at least one key"));
    }

    for key in args.clone() {
      if let Some(value) = self.store.get(key.as_str()).await {
        debug!("Deleting key {} with value {:?}", key, value);
        self.store.delete(key.as_str()).await;
      }
    }

    Ok(Value::Integer(args.len() as i64))
  }
}
