#![warn(rust_2018_idioms)]

extern crate tokio;
extern crate tokio_util;
mod command;
mod db;
mod response;
mod parser;
mod byte_utils;

use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::{Framed, BytesCodec};
use futures::SinkExt;

use std::env;
use std::error::Error;


use crate::db::Database;
use crate::command::Command;
use bytes::{BytesMut, BufMut, Buf};


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
            Ok((mut socket, client_addr)) => {
                println!("Establing connection with {:?}", client_addr);
                let db = db.clone();
                tokio::spawn(async move {
                    loop {
                        let mut buf = [0u8; 1024];
                        let mut bytes_mut = BytesMut::new();

                        while let Ok(n) = socket.read(&mut buf).await {
                            match n {
                                0 => {
                                    println!("Socket closed");
                                    return
                                },
                                1024=> {
                                    println!("Received {} bytes", n);
                                    bytes_mut.put_slice(&buf);
                                },
                                n => {
                                    println!("Received {} bytes", n);
                                    bytes_mut.put_slice(&buf[0..n]);
                                    let response = Command::handle(bytes_mut.bytes(), &db);
                                    let response_bytes = response.serialize();
                                    if let Err(e) = socket.write_all(response_bytes.bytes()).await {
                                        println!("error on sending response; error = {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                });
            }
            Err(e) => println!("error accepting socket; error = {:?}", e),
        }
    }
}