use std::sync::Arc;
use crate::db::Database;
use crate::response::Response;

pub enum Command {
    Get { key: String },
    Set { key: String, value: String },
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
                Some(value) => Response::Value {
                    key,
                    value: value.clone(),
                },
                None => Response::Error {
                    msg: format!("no key {}", key),
                },
            },
            Command::Set { key, value } => {
                let previous = db.insert(key.clone(), value.clone());
                Response::Set {
                    key,
                    value,
                    previous,
                }
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
                    value: value.to_string(),
                })
            }
            Some(cmd) => Err(format!("unknown command: {}", cmd)),
            None => Err("empty input".into()),
        }
    }
}

