use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::resp::value::Value;

// Make sure KvSet and KvHashMap are public
pub type KvSet = HashSet<String>;
pub type KvHashMap = HashMap<String, Value>;

#[derive(Debug)]
pub enum Entities {
    Set(Arc<Mutex<KvSet>>),
    HashMap(Arc<Mutex<KvHashMap>>),
    HashSet,
    List,
    LinkedList,
    Queue,
}
