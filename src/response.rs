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
    pub fn serialize(&self) -> Vec<u8> {
        match *self {
            Response::Value { ref value } => value.clone(),
            Response::Stored => b"STORED".to_vec(),
            Response::NotFoundError => b"NOT_FOUND".to_vec(),
            Response::ServerError => b"SERVER_ERROR".to_vec(),
            Response::Error { ref msg } => msg.as_bytes().to_vec(),
        }.to_owned()
    }
}