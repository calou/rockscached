pub mod configuration;


use tokio::prelude::*;
use tokio::net::TcpListener;
use std::str;
use std::fs::File;
use std::io::prelude::*;
use log::info;

use std::borrow::Borrow;
use crate::configuration::Configuration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("server.yml")?;
    let config: Configuration = serde_yaml::from_reader(&file).unwrap();


    let tcp_uri = format!("127.0.0.1:{}", config.port.borrow());
    info!("Server listening on {}", tcp_uri);

    let mut listener = TcpListener::bind(&tcp_uri).await?;
    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => {
                        println!("Message received {}", str::from_utf8(&buf).unwrap());
                        n
                    },
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
