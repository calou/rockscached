use std::sync::{Mutex, Arc};
use rocksdb::{DB, Options, DBCompressionType, Error};
use crate::response::Response;
use bytes::{BytesMut, BufMut, Buf};
use std::time::SystemTime;
use byteorder::{ByteOrder, BigEndian};
use crate::byte_utils::{convert_bytes_to_u64, u64_to_bytes};

#[derive(Debug)]
struct DatabaseHolder {
    rocksdb: DB,
    cas: u64,
}

impl DatabaseHolder {
    fn increment_cas(& mut self) -> u64 {
        self.cas +=1;
        self.cas
    }
}

pub struct Database {
    mutex: Mutex<DatabaseHolder>,
}

impl Database {
    pub fn open(path: &str) -> Arc<Database> {
        let mut db_opts = Options::default();
        db_opts.set_compression_type(DBCompressionType::Lz4);
        db_opts.set_max_write_buffer_number(16);
        db_opts.create_if_missing(true);
        let initial_db = DB::open(&db_opts, path).unwrap();
        let dh = DatabaseHolder {rocksdb: initial_db, cas: 0};
        Arc::new(Database {
            mutex: Mutex::new(dh),
        })
    }

    pub fn get(&self, keys: Vec<&[u8]>, include_cas: bool) -> Response {
        let mut bytes_mut = BytesMut::new();
        let dh = self.mutex.lock().unwrap();
        let rocksdb = &dh.rocksdb;
        for key in keys {
            match rocksdb.get(key) {
                Ok(Some(value)) => append_get_response(key, &value, &mut bytes_mut,include_cas),
                _ => ()
            };
        }
        finish_get_response(&mut bytes_mut)
    }

    pub fn delete(&self, key: &[u8]) -> Response {
        let dh = self.mutex.lock().unwrap();
        let rocksdb = &dh.rocksdb;
        match rocksdb.delete(key) {
            Ok(()) => Response::Stored,
            Err(_) => Response::NotFoundError
        }
    }

    fn get_raw_value(&self, key: &[u8]) -> Option<Vec<u8>> {
        let dh = self.mutex.lock().unwrap();
        let rocksdb = &dh.rocksdb;
        match rocksdb.get(key) {
            Ok(Some(value)) => Some(value[18..].to_vec()),
            _ => None,
        }
    }

    pub fn insert(&self, key: &[u8], flags: u32, ttl: u64, value: &[u8]) -> Response {
        let deadline = current_second() + ttl;
        self.insert_with_deadline(key, flags, deadline, value)
    }

    pub fn insert_if_not_present(&self, key: &[u8], flags: u32, ttl: u64, value: &[u8]) -> Response {
        match self.get_raw_value(key) {
            Some(_) => Response::ServerError,
            _ => self.insert(key, flags, ttl, value)
        }
    }

    pub fn append(&self, key: &[u8], flags: u32, ttl: u64, value: &[u8]) -> Response {
        let f = |original: Vec<u8>, appendage: &[u8]| -> Vec<u8> {
            let mut bytes_mut = BytesMut::with_capacity(original.len() + appendage.len());
            bytes_mut.put_slice(&original);
            bytes_mut.put_slice(appendage);
            bytes_mut.bytes().to_vec()
        };
        self.update_combined(key, flags, ttl, value, f)
    }

    pub fn prepend(&self, key: &[u8], flags: u32, ttl: u64, value: &[u8]) -> Response {
        let f = |original: Vec<u8>, appendage: &[u8]| -> Vec<u8> {
            let mut bytes_mut = BytesMut::with_capacity(original.len() + appendage.len());
            bytes_mut.put_slice(appendage);
            bytes_mut.put_slice(&original);
            bytes_mut.bytes().to_vec()
        };
        self.update_combined(key, flags, ttl, value, f)
    }

    fn insert_with_deadline(&self, key: &[u8], flags: u32, deadline: u64, value: &[u8]) -> Response {
        let deadline_bytes = &u64::to_be_bytes(deadline);
        let mut flag_bytes = [0; 4];
        BigEndian::write_u32(&mut flag_bytes, flags);
        self.insert_raw(key, deadline_bytes, &mut flag_bytes, value)
    }

    fn insert_raw(&self, key: &[u8], deadline_bytes: &[u8], flag_bytes: &[u8], value: &[u8]) -> Response {
        let mut dh = self.mutex.lock().unwrap();
        let cas  = dh.increment_cas();
        let mut bytes_mut = BytesMut::with_capacity(18 + value.len());
        bytes_mut.put_slice(deadline_bytes);
        bytes_mut.put_u64(cas);
        bytes_mut.put_slice(&flag_bytes);
        bytes_mut.put_slice(value);
        let rocksdb = &dh.rocksdb;

        match rocksdb.put(key, bytes_mut.bytes()) {
            Ok(_) => Response::Stored,
            _ => Response::ServerError

        }
    }

    fn update_combined<'a, I>(&self, key: &[u8], flags: u32, ttl: u64, value: &'a [u8], f: I) -> Response
        where I: Fn(Vec<u8>, &'a [u8]) -> Vec<u8>
    {
        match self.get_raw_value(key) {
            Some(original) => {
                self.insert(key, flags, ttl, &f(original, value))
            }
            _ => Response::NotStored
        }
    }

    pub fn increment(&self, key: &[u8], increment: u64) -> Response {
        self.update_number(key, increment, |a, b| { a + b })
    }

    pub fn decrement(&self, key: &[u8], increment: u64) -> Response {
        self.update_number(key, increment, |a, b| { a - b })
    }

    fn update_number<'a, I>(&self, key: &[u8], increment: u64, f: I) -> Response
        where I: Fn(u64, u64) -> u64
    {
        match self.get_record(key) {
            Ok(Some(value)) => {
                let expiration = BigEndian::read_u64(&value[0..8]);

                if expiration < current_second() {
                    Response::NotFoundError
                } else {
                    match convert_bytes_to_u64(&value[20..]) {
                        Ok(stored_value) => {
                            let updated_value = f(stored_value, increment);
                            let new_value_bytes = u64_to_bytes(updated_value);
                            match self.insert_raw(key, &value[0..8], &value[16..20], &new_value_bytes) {
                                Response::Stored => {
                                    let mut bytes_mut = BytesMut::with_capacity(new_value_bytes.len() + 2);
                                    bytes_mut.put_slice(&new_value_bytes);
                                    bytes_mut.put_slice(b"\r\n");
                                    Response::Value { value: bytes_mut.to_vec() }
                                }
                                _ => Response::ServerError
                            }
                        }
                        Err(e) => {
                            println!("An error occured {}", e);
                            Response::Error {
                                msg: Box::new(String::from("CLIENT_ERROR cannot increment or decrement non-numeric value\r\n"))
                            }
                        }
                    }
                }
            }
            _ => Response::NotStored
        }
    }

    fn get_record(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let dh = self.mutex.lock().unwrap();
        let rocksdb = &dh.rocksdb;
        rocksdb.get(key)
    }
}

fn finish_get_response(bytes_mut: &mut BytesMut) -> Response {
    bytes_mut.put_slice(b"END\r\n");
    Response::Value {
        value: bytes_mut.bytes().to_vec(),
    }
}

fn append_get_response(key: &[u8], value: &[u8], bytes_mut: &mut BytesMut, include_cas:bool) {
    let expiration = BigEndian::read_u64(&value[0..8]);
    if expiration > current_second() {
        let cas = match include_cas {
            true => Some(BigEndian::read_u64(&value[8..16])),
            _ => None
        };
        append_raw_get_response(key, cas, &value[16..20], &value[20..], bytes_mut);
    }
}

fn append_raw_get_response(key: &[u8], cas: Option<u64>, flag_bytes: &[u8], data_bytes: &[u8], bytes_mut: &mut BytesMut) {
    let flag = BigEndian::read_u32(flag_bytes);
    let length = data_bytes.len() as u32;
    bytes_mut.put_slice(b"VALUE ");
    bytes_mut.put_slice(key);
    bytes_mut.put_slice(b" ");
    bytes_mut.put_slice(&flag.to_string().into_bytes());
    bytes_mut.put_slice(b" ");
    bytes_mut.put_slice(&length.to_string().into_bytes());
    match cas {
        Some(n) => {
            bytes_mut.put_slice(b" ");
            bytes_mut.put_slice(&n.to_string().into_bytes());
        }
        _ => ()
    }
    bytes_mut.put_slice(b"\r\n");
    bytes_mut.put_slice(data_bytes);
    bytes_mut.put_slice(b"\r\n");
}

fn current_second() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}


unsafe impl Send for Database {}

unsafe impl Sync for Database {}