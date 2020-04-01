use std::sync::{Mutex, Arc, MutexGuard};
use rocksdb::{DB};

pub struct Database {
    pub map: Mutex<DB>,
}

impl Database {
    pub fn open(path: &str) -> Arc<Database> {
        let initial_db = DB::open_default(path).unwrap();
        initial_db.put(b"foo", b"bar").unwrap();
        Arc::new(Database {
            map: Mutex::new(initial_db),
        })
    }
}

pub fn insert( _db: MutexGuard<'_, DB>, _key: &'static[u8], _flags: &'static[u8],  _expiration_timestamp: u64, _value: &'static[u8]) {
    // TODO
}