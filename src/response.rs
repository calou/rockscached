use std::borrow::Borrow;
use std::ops::Deref;

/// Responses to the `Request` commands above
pub enum Response {
    Value {
        value: Vec<u8>,
    },
    Set,
    NotFoundError,
    Error {
        msg: String,
    },
}

impl Response {
    pub fn serialize(&self) -> Vec<u8> {
        match *self {
            Response::Value { ref value } => value.clone(),
            Response::Set => "STORED".as_bytes().to_vec(),
            Response::NotFoundError => "NOT FOUND".as_bytes().to_vec(),
            Response::Error { ref msg } => msg.as_bytes().to_vec(),
        }.to_owned()
    }
}
