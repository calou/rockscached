use std::sync::Arc;
use crate::db::Database;
use crate::response::Response;
use crate::parser::parse;

#[derive(PartialEq, Debug)]
pub enum Command<'a> {
    MGet { key: &'a [u8] },
    MSet { key: &'a [u8], flags: &'a [u8], ttl: u64, value: &'a [u8] },
    MAdd { key: &'a [u8], flags: &'a [u8], ttl: u64, value: &'a [u8] },
    MAppend { key: &'a [u8], flags: &'a [u8], ttl: u64, value: &'a [u8] },
    MPrepend { key: &'a [u8], flags: &'a [u8], ttl: u64, value: &'a [u8] },
}



impl<'a> Command<'a> {
    pub fn handle(line: &[u8], db: &Arc<Database>) -> Response {
        let request = match parse(line) {
            Ok(req) => req,
            Err(e) => return Response::Error { msg: e },
        };

        let db = db.clone();
        match request {
            Command::MGet { key } => db.get(key),
            Command::MSet { key, flags, ttl, value } => db.insert(key, flags, ttl, value),
            _ => Response::ServerError
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
}

