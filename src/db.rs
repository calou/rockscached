use std::sync::{Mutex, Arc};
use rocksdb::{DB, Options, DBCompressionType};
use crate::response::Response;
use bytes::{BytesMut, BufMut, Buf};
use std::time::SystemTime;
use byteorder::{ByteOrder, BigEndian};

pub struct Database {
    pub map: Mutex<DB>,
}

impl Database {
    pub fn open(path: &str) -> Arc<Database> {
        let mut db_opts = Options::default();
        db_opts.set_compression_type(DBCompressionType::Lz4);
        db_opts.set_max_write_buffer_number(16);
        db_opts.create_if_missing(true);
        let initial_db = DB::open(&db_opts, path).unwrap();
        Arc::new(Database {
            map: Mutex::new(initial_db),
        })
    }

    pub fn get(&self, key: &[u8]) -> Response {
        let rocksdb = self.map.lock().unwrap();
        match rocksdb.get(key) {
            Ok(Some(value)) => format_get_response(key, &value),
            Ok(None) => Response::NotFoundError,
            Err(e) => Response::Error {
                msg: format!("Error {}", e),
            },
        }
    }

    pub fn insert(&self, key: &[u8], flags: u32, ttl: u64, value: &[u8]) -> Response {
        let mut bytes_mut = BytesMut::with_capacity(12 + value.len());
        let deadline = current_second() + ttl;
        bytes_mut.put_slice(&u64::to_be_bytes(deadline));

        let mut flag_bytes= [0; 4];
        BigEndian::write_u32(&mut flag_bytes, flags);
        bytes_mut.put_slice(&flag_bytes);
        bytes_mut.put_slice(value);
        let rocksdb = self.map.lock().unwrap();
        match rocksdb.put(key, bytes_mut.bytes()) {
            Ok(()) => Response::Stored,
            _ => Response::ServerError,
        }
    }
}

fn format_get_response(key: &[u8], value: &[u8]) -> Response {
    let expiration = BigEndian::read_u64(&value[0..8]);
    if expiration < current_second() {
        Response::NotFoundError
    } else {
        let mut resp = BytesMut::new();
        let length = value.len() as u32;
        let flag = BigEndian::read_u32(&value[8..12]);

        resp.put_slice(b"VALUE ");
        resp.put_slice(key);
        resp.put_slice(b" ");
        resp.put_slice(&flag.to_string().into_bytes());
        resp.put_slice(b" ");
        resp.put_slice(&length.to_string().into_bytes());
        resp.put_slice(b"\r\n");
        resp.put_slice(&value[12..]);
        resp.put_slice(b"\r\nEND\r\n");
        Response::Value {
            value: resp.bytes().to_vec(),
        }
    }
}

fn current_second() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}


unsafe impl Send for Database {}

unsafe impl Sync for Database {}