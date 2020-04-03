use bytes::Bytes;

#[derive(PartialEq)]
pub enum Response {
    Value {
        value: Vec<u8>,
    },
    Stored,
    NotFoundError,
    ServerError,
    Error {
        msg: String,
    },
}

impl Response {
    pub fn serialize(&self) -> Bytes {

        match *self {
            Response::Value { ref value } => Bytes::from(value.clone()),
            Response::Stored => Bytes::from("STORED\r\n"),
            Response::NotFoundError => Bytes::from("END\r\n"),
            Response::ServerError => Bytes::from("SERVER_ERROR"),
            _ => Bytes::from("SERVER_ERROR"),
        }.to_owned()
    }
}