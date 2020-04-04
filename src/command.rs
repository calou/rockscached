use std::sync::Arc;
use crate::db::Database;
use crate::response::Response;
use crate::parser::parse;

#[derive(PartialEq, Debug)]
pub enum Command<'a> {
    Get { key: &'a [u8] },
    Set { key: &'a [u8], flags: u32, ttl: u64, value: &'a [u8] },
    Add { key: &'a [u8], flags: u32, ttl: u64, value: &'a [u8] },
    Append { key: &'a [u8], flags: u32, ttl: u64, value: &'a [u8] },
    Prepend { key: &'a [u8], flags: u32, ttl: u64, value: &'a [u8] },
}

impl<'a> Command<'a> {
    pub fn handle(line: &[u8], db: &Arc<Database>) -> Response {
        let request = match parse(line) {
            Ok(req) => req,
            Err(e) => return Response::Error { msg: e },
        };

        let db = db.clone();
        match request {
            Command::Get { key } => db.get(key),
            Command::Set { key, flags, ttl, value } => db.insert(key, flags, ttl, value),
            Command::Add { key, flags, ttl, value } => db.insert_new(key, flags, ttl, value),
            Command::Append { key, flags, ttl, value } => db.append(key, flags, ttl, value),
            Command::Prepend { key, flags, ttl, value } => db.prepend(key, flags, ttl, value),
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
}

