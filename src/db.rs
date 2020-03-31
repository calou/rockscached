use std::sync::{Mutex, Arc};
use rocksdb::{DB, Options};

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