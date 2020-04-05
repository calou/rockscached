use bytes::Bytes;

#[derive(PartialEq)]
pub enum Response {
    Value {
        value: Vec<u8>,
    },
    Stored,
    NotStored,
    NotFoundError,
    ServerError,
    Error {
        msg: Box<String>,
    },
}

impl Response {
    pub fn serialize(&self) -> Bytes {
        match *self {
            Response::Value { ref value } => Bytes::from(value.clone()),
            Response::Stored => Bytes::from("STORED\r\n"),
            Response::NotFoundError => Bytes::from("END\r\n"),
            Response::ServerError => Bytes::from("SERVER_ERROR\r\n"),
            Response::NotStored => Bytes::from("NOT_STORED\r\n"),
            _ => Bytes::from("ERROR\r\n"),
        }
    }
}