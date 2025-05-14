use super::Entity;
use crate::resp::value::Value;
use std::collections::HashMap;

pub struct KvHashMap {
  data: HashMap<String, String>,
}

impl Entity for KvHashMap {
  fn entity_type(&self) -> &'static str {
    "hashmap"
  }

  fn get(&self, key: &str) -> Option<Value> {
    self
      .data
      .get(key)
      .map(|val| Value::SimpleString(val.clone()))
  }

  fn set(&mut self, key: &str, value: Value) {
    if let Value::SimpleString(val) = value {
      self.data.insert(key.to_string(), val);
    }
  }

  fn delete(&mut self, key: &str) -> Option<Value> {
    self.data.remove(key).map(|val| Value::SimpleString(val))
  }
}
