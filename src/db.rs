use std::sync::{Mutex, Arc};
use rocksdb::{DB, Options, ColumnFamilyDescriptor, DBCompressionType, ColumnFamily};
use crate::response::Response;
use bytes::{BytesMut, BufMut, Buf};
use std::time::SystemTime;
use crate::byte_utils::{bytes_to_u64, u64_to_bytes};

const EXPIRATION_PREFIX: &[u8] = b"**e**";

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
        match rocksdb.get(get_prefixed_key(EXPIRATION_PREFIX, key.clone())) {
            Ok(Some(exp)) => {
                let expiration = bytes_to_u64(&exp);
                if expiration < current_second() {
                    Response::NotFoundError
                } else {
                    match rocksdb.get(key.clone()) {
                        Ok(Some(value)) => {
                            let mut resp = BytesMut::new();

                            let length = value.len() as u32;

                            resp.put_slice(b"VALUE ");
                            resp.put_slice(key.clone());
                            resp.put_slice(b" 0 ");
                            resp.put_slice(&length.to_string().into_bytes());
                            resp.put_slice(b"\r\n");
                            resp.put_slice(&value.clone());
                            resp.put_slice(b"\r\nEND\r\n");
                            Response::Value {
                                value: resp.bytes().to_vec(),
                            }
                        },
                        Ok(None) => Response::NotFoundError,
                        Err(e) => Response::Error {
                            msg: format!("Error {}", e),
                        },
                    }
                }
            }
            Ok(None) => Response::NotFoundError,
            Err(e) => Response::Error {
                msg: format!("Error {}", e),
            },
        }
    }

    pub fn insert(&self, key: &[u8], flags: &[u8], ttl: u64, value:&[u8]) -> Response {
        let exp_key = get_prefixed_key(EXPIRATION_PREFIX, key.clone());
        let deadline = current_second() + ttl;

        let rocksdb = self.map.lock().unwrap();
        match rocksdb.put(exp_key, u64_to_bytes(deadline)) {
            Ok(()) => (),
            _ => return Response::ServerError,
        }

        match rocksdb.put(key.clone(), value.clone()) {
            Ok(()) => Response::Stored,
            _ => Response::ServerError,
        }
    }
}


fn get_prefixed_key(prefix: &[u8], key: &[u8]) -> Vec<u8> {
    let mut bytes_mut = BytesMut::with_capacity(key.to_owned().len() + prefix.len());
    bytes_mut.put(prefix);
    bytes_mut.put(key);
    bytes_mut.to_vec()
}

fn current_second() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}


unsafe impl Send for Database {}
unsafe impl Sync for Database {}