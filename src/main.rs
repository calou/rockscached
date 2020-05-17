#![warn(rust_2018_idioms)]

mod command;
mod db;
mod response;
mod parser;
mod byte_utils;
use std::env;
use std::error::Error;
use log::{info, error};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


use crate::db::Database;
use crate::command::Command;
use bytes::{BytesMut, BufMut, Buf};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let mut listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    let db = Database::open("/tmp/rocksdb.db");

    loop {
        match listener.accept().await {
            Ok((mut socket, client_addr)) => {
                info!("Establing connection with {:?}", client_addr);
                let db = db.clone();
                tokio::spawn(async move {
                    loop {
                        let mut buf = [0u8; 1024];
                        let mut bytes_mut = BytesMut::new();

                        while let Ok(n) = socket.read(&mut buf).await {
                            match n {
                                0 => {
                                    return;
                                }
                                n => {
                                    bytes_mut.put_slice(&buf[0..n]);
                                    if n < 1024 || (buf[1022] == b'\r' && buf[1023] == b'\n') {
                                        let response = Command::handle(bytes_mut.bytes(), &db);
                                        let response_bytes = response.serialize();
                                        if let Err(e) = socket.write_all(response_bytes.bytes()).await {
                                            error!("error on sending response; error = {:?}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }
            Err(e) => error!("error accepting socket; error = {:?}", e),
        }
    }
}