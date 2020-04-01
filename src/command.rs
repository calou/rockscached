use std::sync::Arc;
use crate::db::{Database};
use crate::response::Response;
use crate::parser::parse;

#[derive(PartialEq, Debug)]
pub enum Command<'a> {
    MGet { key: &'a[u8] },
    MSet { key: &'a[u8], flags:&'a[u8], ttl: u64, value:&'a[u8] },
}

impl<'a> Command<'a> {
    pub fn handle(line: &[u8], db: &Arc<Database>) -> Response {

        let request = match parse(line) {
            Ok(req) => req,
            Err(e) => return Response::Error { msg: e },
        };

        let rocksdb = db.map.lock().unwrap();
        match request {
            Command::MGet { key } => match rocksdb.get(key) {
                Ok(Some(value)) => Response::Value {
                    value: value.clone(),
                },
                Ok(None) => Response::NotFoundError,
                Err(e) => Response::Error {
                    msg: format!("Error {}", e),
                },
            },
            Command::MSet { key, flags: _, ttl: _expiration, value } => {

                match rocksdb.put(key.clone(), value.clone()) {
                    Ok(()) => Response::Stored,
                    _ => Response::ServerError,
                }

            },
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    
}

