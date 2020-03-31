use std::sync::Mutex;
use rocksdb::{DB, Options};

pub struct Database {
    pub map: Mutex<DB>,
}