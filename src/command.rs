use std::sync::Arc;
use crate::db::{Database, insert};
use crate::response::Response;
use crate::command::Command::MemcachedSet;
use crate::parser;

#[derive(PartialEq, Debug)]
pub enum Command {
    Get { key: String },
    Set { key: String, value: Vec<u8> },
    MemcachedSet { key: &'static [u8], flags: &'static [u8], expiration_timestamp: u64, value: &'static [u8] },
}

pub trait MemcachedCommand {
    fn execute(self, db: &Arc<Database>) -> Result<String, String>;
}

#[derive(PartialEq, Debug)]
pub struct MemcachedCommandSet<'a> {
    key: &'a[u8],
    flags: &'a[u8],
    expiration_timestamp: u64,
    value: &'a[u8],
}

impl<'a> MemcachedCommand for MemcachedCommandSet<'a> {
    fn execute(self, db: &Arc<Database>) -> Result<String, String> {
        unimplemented!()
    }
}


impl Command {
    pub fn handle(line: &str, db: &Arc<Database>) -> Response {
        let request = match Command::parse(&line) {
            Ok(req) => req,
            Err(e) => return Response::Error { msg: e },
        };


        let rocksdb = db.map.lock().unwrap();
        match request {
            Command::Get { key } => match rocksdb.get(&key) {
                Ok(Some(value)) => Response::Value {
                    value: value.clone(),
                },
                Ok(None) => Response::NotFoundError,
                Err(e) => Response::Error {
                    msg: format!("Error {}", e),
                },
            },
            Command::Set { key, value } => {
                rocksdb.put(key.clone(), value.clone());
                Response::Stored
            }
            Command::MemcachedSet { key, flags, expiration_timestamp, value } => {
                insert(rocksdb, key, flags, expiration_timestamp, value);
                Response::Stored
            }
        }
    }
    fn parse(input: &str) -> Result<Command, String> {
        let mut parts = input.splitn(3, ' ');
        match parts.next() {
            Some("GET") => {
                let key = parts.next().ok_or("GET must be followed by a key")?;
                if parts.next().is_some() {
                    return Err("GET's key must not be followed by anything".into());
                }
                Ok(Command::Get {
                    key: key.to_string(),
                })
            }
            Some("SET") => {
                let key = match parts.next() {
                    Some(key) => key,
                    None => return Err("SET must be followed by a key".into()),
                };
                let value = match parts.next() {
                    Some(value) => value,
                    None => return Err("SET needs a value".into()),
                };
                Ok(Command::Set {
                    key: key.to_string(),
                    value: value.as_bytes().to_vec(),
                })
            }
            Some(cmd) => Err(format!("unknown command: {}", cmd)),
            None => Err("empty input".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
}

