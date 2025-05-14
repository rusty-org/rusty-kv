use crate::resp::value::Value;

pub mod hmap;
pub mod hset;
pub mod llist;

pub trait Entity: Send + Sync {
  fn entity_type(&self) -> &'static str;
  fn get(&self, key: &str) -> Option<Value>;
  fn set(&mut self, key: &str, value: Value);
  fn delete(&mut self, key: &str) -> Option<Value>;
}
