use std::sync::Mutex;
use std::collections::HashMap;

pub struct Database {
    pub map: Mutex<HashMap<String, String>>,
}