use std::sync::Arc;
use crate::db::Database;
use crate::response::Response;

pub enum Command {
    Get { key: String },
    Set { key: String, value: Vec<u8> },
}

impl Command {
    pub fn handle(line: &str, db: &Arc<Database>) -> Response {
        let request = match Command::parse(&line) {
            Ok(req) => req,
            Err(e) => return Response::Error { msg: e },
        };

        let mut db = db.map.lock().unwrap();
        match request {
            Command::Get { key } => match db.get(&key) {
                Ok(Some(value)) => Response::Value {
                    value: value.clone(),
                },
                Ok(None) => Response::Error {
                    msg: format!("no key {}", key),
                },
                Err(e) =>  Response::Error {
                    msg: format!("Erro {}", e),
                },
            },
            Command::Set { key, value } => {
                db.put(key.clone(), value.clone());
                Response::Set
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

