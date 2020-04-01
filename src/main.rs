#![warn(rust_2018_idioms)]
mod command;
mod db;
mod response;
mod parser;

use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, BytesCodec};
use bytes::{BytesMut, BufMut};
use futures::SinkExt;

use std::env;
use std::error::Error;


use crate::db::Database;
use crate::command::Command;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    let db = Database::open("/tmp/rocksdb.db");

    loop {
        match listener.accept().await {
            Ok((socket, client_addr)) => {
                println!("Establing connection with {:?}", client_addr);
                let db = db.clone();
                tokio::spawn(async move {
                    let mut framed = Framed::new(socket, BytesCodec::new());

                    while let Some(result) = framed.next().await {
                        match result {
                            Ok(line) => {
                                let response = Command::handle(&line, &db);
                                let response_bytes = response.serialize();
                                let mut bytes = BytesMut::new();
                                bytes.put(&response_bytes[..]);
                                bytes.put_slice(b"\r\n");
                                if let Err(e) = framed.send(bytes.freeze()).await {
                                    println!("error on sending response; error = {:?}", e);
                                }
                            }
                            Err(e) => {
                                println!("error on decoding from socket; error = {:?}", e);
                            }
                        }
                    }
                });
            }
            Err(e) => println!("error accepting socket; error = {:?}", e),
        }
    }
}