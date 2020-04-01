use std::borrow::Borrow;
use std::ops::Deref;

#[derive(PartialEq)]
pub enum Response {
    Value {
        value: Vec<u8>,
    },
    Stored,
    NotFoundError,
    Error {
        msg: String,
    },
}

impl Response {
    pub fn serialize(&self) -> Vec<u8> {
        match *self {
            Response::Value { ref value } => value.clone(),
            Response::Stored => "STORED\r\n".as_bytes().to_vec(),
            Response::NotFoundError => "NOT_FOUND\r\n".as_bytes().to_vec(),
            Response::Error { ref msg } => msg.as_bytes().to_vec(),
        }.to_owned()
    }
}