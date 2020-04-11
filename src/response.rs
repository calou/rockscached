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
    NotImplemented,
    Error {
        msg: Box<String>,
    },
}

impl Response {
    pub fn serialize(&self) -> Bytes {
        match &*self {
            Response::Value { ref value } => Bytes::from(value.clone()),
            Response::Stored => Bytes::from("STORED\r\n"),
            Response::NotFoundError => Bytes::from("END\r\n"),
            Response::ServerError => Bytes::from("SERVER_ERROR\r\n"),
            Response::NotStored => Bytes::from("NOT_STORED\r\n"),
            Response::NotImplemented => Bytes::from("NOT_IMPLEMENTED\r\n"),
            Response::Error {msg} => {
                println!("{}", msg);
                Bytes::from("ERROR\r\n")
            },
        }
    }
}