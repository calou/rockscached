#![warn(rust_2018_idioms)]


use std::error::Error;
use log::{info, error};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use clap::{Arg, App};
use bytes::{BytesMut, BufMut, Buf};

use rockscached_db::db::Database;
use rockscached_db::command::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("RocksCached")
        .version("0.1.0")
        .author("Sébastien G. <gruchet@gmail.com>")
        .about("RocksDB backed Memcached replacement")
        .arg(Arg::with_name("address")
            .short("a")
            .long("address")
            .value_name("host:port")
            .help("The socket address to listen to")
            .default_value("127.0.0.1:8080")
            .takes_value(true))
        .arg(Arg::with_name("db_dir")
            .short("d")
            .long("db_dir")
            .value_name("directory")
            .help("The directory where the data will be stored")
            .default_value("/tmp/rocksdb")
            .takes_value(true))
        .get_matches();

    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let addr = matches.value_of("address").unwrap_or("127.0.0.1:8080");

    let mut listener = TcpListener::bind(&addr).await?;
    let database_directory = matches.value_of("db_dir").unwrap_or("/tmp/rocksdb");
    info!("Listening on: {}", addr);
    info!("Storing data in {}", database_directory);
    let db = Database::open(database_directory);
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