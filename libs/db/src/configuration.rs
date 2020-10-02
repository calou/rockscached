use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    pub port: i16,
    pub storage_path: String,
}
