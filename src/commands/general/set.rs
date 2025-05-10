use crate::{resp::value::Value, storage::memory::MemoryStore, storage::memory::Store};
use anyhow::{Result, anyhow};
use log::debug;

pub struct SetCommand;

impl SetCommand {
  pub async fn execute(mut args: Vec<String>, store: MemoryStore) -> Result<Value> {
    if args.len() < 2 {
      return Err(anyhow!("SET requires a key and a value"));
    }

    let key = args[0].to_owned();
    let value = args[1].to_owned();

    // @NOTE Find any other optional arguments
    // Such as EX, PX, NX, XX
    while args.len() > 2 {
      let arg = args.remove(2);
      match arg.to_uppercase().as_str() {
        "EX" => {
          // Handle expiration in seconds
          if let Some(expiration) = args.get(2) {
            debug!("Setting expiration to {} seconds", expiration);
            args.remove(2);
          }
        }
        "PX" => {
          // Handle expiration in milliseconds
          if let Some(expiration) = args.get(2) {
            debug!("Setting expiration to {} milliseconds", expiration);
            args.remove(2);
          }
        }
        "NX" => {
          // Handle only set if not exists
          // Logic for NX goes here
        }
        "XX" => {
          // Handle only set if exists
          // Logic for XX goes here
        }
        _ => {}
      }
    }

    // Set the value in the store
    store
      .set(key.as_str(), Value::SimpleString(value.clone()))
      .await;
    debug!("Set key {} to value {}", key, value);

    Ok(Value::SimpleString("OK".to_string()))
  }
}
