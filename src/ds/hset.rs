use super::Entity;
use crate::resp::value::Value;
use std::collections::HashSet;

pub struct KvSet {
  data: HashSet<String>,
}

impl KvSet {
  pub fn new() -> Self {
    Self {
      data: HashSet::new(),
    }
  }
}

impl Entity for KvSet {
  fn entity_type(&self) -> &'static str {
    "set"
  }

  fn get(&self, key: &str) -> Option<Value> {
    if self.data.contains(key) {
      Some(Value::SimpleString(key.to_string()))
    } else {
      None
    }
  }

  fn set(&mut self, _key: &str, value: Value) {
    if let Value::SimpleString(val) = value {
      self.data.insert(val);
    }
  }

  fn delete(&mut self, key: &str) -> Option<Value> {
    if self.data.remove(key) {
      Some(Value::SimpleString(key.to_string()))
    } else {
      None
    }
  }
}
