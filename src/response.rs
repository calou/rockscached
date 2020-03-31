/// Responses to the `Request` commands above
pub enum Response {
    Value {
        key: String,
        value: Vec<u8>,
    },
    Set {
        key: String,
        value: Vec<u8>,
    },
    Error {
        msg: String,
    },
}

impl Response {
    pub fn serialize(&self) -> String {
        match *self {
            Response::Value { ref key, ref value } => format!("{} = {:?}", key, value),
            Response::Set {
                ref key,
                ref value,
            } => format!("set {} = `{:?}`", key, value),
            Response::Error { ref msg } => format!("error: {}", msg),
        }
    }
}
