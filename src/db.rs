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

    pub fn delete(&self, key: &[u8]) -> Response {
        let rocksdb = self.map.lock().unwrap();
        match rocksdb.delete(key) {
            Ok(()) => Response::Stored,
            Err(_) => Response::NotFoundError
        }
    }

    fn get_raw_value(&self, key: &[u8]) -> Option<Vec<u8>> {
        let rocksdb = self.map.lock().unwrap();
        match rocksdb.get(key) {
            Ok(Some(value)) => Some(value[12..].to_vec()),
            _ => None,
        }
    }

    pub fn insert(&self, key: &[u8], flags: u32, ttl: u64, value: &[u8]) -> Response {
        let deadline = current_second() + ttl;
        self.insert_with_deadline(key, flags, deadline, value)
    }

    fn insert_with_deadline(&self, key: &[u8], flags: u32, deadline: u64, value: &[u8]) -> Response {
        let mut bytes_mut = BytesMut::with_capacity(12 + value.len());
        bytes_mut.put_slice(&u64::to_be_bytes(deadline));

        let mut flag_bytes = [0; 4];
        BigEndian::write_u32(&mut flag_bytes, flags);
        bytes_mut.put_slice(&flag_bytes);
        bytes_mut.put_slice(value);
        let rocksdb = self.map.lock().unwrap();
        match rocksdb.put(key, bytes_mut.bytes()) {
            Ok(()) => Response::Stored,
            _ => Response::ServerError,
        }
    }

    pub fn insert_if_not_present(&self, key: &[u8], flags: u32, ttl: u64, value: &[u8]) -> Response {
        match self.get_raw_value(key) {
            Some(_) => Response::ServerError,
            _ => self.insert(key, flags, ttl, value)
        }
    }
    
    fn update_combined<'a, I>(&self, key: &[u8], flags: u32, ttl: u64, value: &'a[u8], f: I) -> Response
        where I: Fn(Vec<u8>, &'a[u8]) -> Vec<u8>
    {
        match self.get_raw_value(key) {
            Some(original) => {
                self.insert(key, flags, ttl, &f(original, value))
            }
            _ => Response::NotStored
        }
    }

    pub fn append(&self, key: &[u8], flags: u32, ttl: u64, value: &[u8]) -> Response {
        let f = |original: Vec<u8>, appenditure: &[u8]| -> Vec<u8> {
            let mut bytes_mut = BytesMut::with_capacity(original.len() + appenditure.len());
            bytes_mut.put_slice(&original);
            bytes_mut.put_slice(appenditure);
            bytes_mut.bytes().to_vec()
        };
        self.update_combined(key, flags, ttl, value, f)
    }

    pub fn prepend(&self, key: &[u8], flags: u32, ttl: u64, value: &[u8]) -> Response {
        let f = |original: Vec<u8>, appenditure: &[u8]| -> Vec<u8> {
            let mut bytes_mut = BytesMut::with_capacity(original.len() + appenditure.len());
            bytes_mut.put_slice(appenditure);
            bytes_mut.put_slice(&original);
            bytes_mut.bytes().to_vec()
        };
        self.update_combined(key, flags, ttl, value, f)
    }
}

fn format_get_response(key: &[u8], value: &[u8]) -> Response {
    let expiration = BigEndian::read_u64(&value[0..8]);
    if expiration < current_second() {
        Response::NotFoundError
    } else {
        format_raw_get_response(key, &value[12..], &value[8..12])
    }
}

fn format_raw_get_response(key: &[u8], data_bytes: &[u8], flag_bytes: &[u8]) -> Response {
    let mut resp = BytesMut::new();
    let flag = BigEndian::read_u32(flag_bytes);
    let length = data_bytes.len() as u32;
    resp.put_slice(b"VALUE ");
    resp.put_slice(key);
    resp.put_slice(b" ");
    resp.put_slice(&flag.to_string().into_bytes());
    resp.put_slice(b" ");
    resp.put_slice(&length.to_string().into_bytes());
    resp.put_slice(b"\r\n");
    resp.put_slice(data_bytes);
    resp.put_slice(b"\r\nEND\r\n");
    Response::Value {
        value: resp.bytes().to_vec(),
    }
}

fn current_second() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}


unsafe impl Send for Database {}

unsafe impl Sync for Database {}