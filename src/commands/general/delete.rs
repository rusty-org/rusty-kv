use anyhow::Result;
use anyhow::anyhow;
use log::debug;

use crate::resp::value::Value;
use crate::storage::memory::MemoryStore;
use crate::storage::memory::Store;

pub struct DeleteCommand;

impl DeleteCommand {
  pub async fn execute(args: Vec<String>, store: MemoryStore) -> Result<Value> {
    if args.is_empty() {
      return Err(anyhow!("DEL requires at least one key"));
    }

    for key in args.clone() {
      if let Some(value) = store.get(key.as_str()).await {
        debug!("Deleting key {} with value {:?}", key, value);
        store.delete(key.as_str()).await;
      }
    }

    Ok(Value::Integer(args.len() as i64))
  }
}
