use std::sync::Arc;
use crate::db::{Database, insert};
use crate::response::Response;
use crate::parser;

#[derive(PartialEq, Debug)]
pub enum Command<'a> {
    Get { key: String },
    Set { key: String, value: Vec<u8> },
    MGet { key: &'a[u8] },
    MSet { key: &'a[u8], flags:&'a[u8], ttl: u64, value:&'a[u8] },
}

impl<'a> Command<'a> {
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
            },
            Command::MGet { key } => match rocksdb.get(key) {
                Ok(Some(value)) => Response::Value {
                    value: value.clone(),
                },
                Ok(None) => Response::NotFoundError,
                Err(e) => Response::Error {
                    msg: format!("Error {}", e),
                },
            },
            Command::MSet { key, flags, ttl: expiration, value } => {
                rocksdb.put(key.clone(), value.clone());
                Response::Stored
            },
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

