use std::sync::{Arc, Mutex};

use crate::ds::{hmap::KvHashMap, hset::KvSet};

pub enum Entities {
  Set(Arc<Mutex<KvSet>>),
  HashMap(Arc<Mutex<KvHashMap>>),
  HashSet,
  List,
  LinkedList,
  Queue,
}
