use crate::resp::value::Value;

pub mod hmap;
pub mod hset;

pub trait Entity {
    fn entity_type(&self) -> &'static str;
    fn get(&self, key: &str) -> Option<Value>;
    fn set(&mut self, key: &str, value: Value);
    fn delete(&mut self, key: &str) -> Option<Value>;
}
