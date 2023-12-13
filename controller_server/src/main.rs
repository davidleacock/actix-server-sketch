use std::io::{Error, ErrorKind};
use actix_web::{App, HttpResponse, HttpServer, web};
use serde::Deserialize;
use serde_json;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;
use tokio::signal;

#[derive(Deserialize, Clone, Debug)]
struct Sensor {
    id: Uuid,
    name: String,
}

async fn tcp_handler(mut stream: TcpStream) -> std::io::Result<()> {
    let (reader, _writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut buffer = Vec::new();

    reader.read_until(b'\n', &mut buffer).await?;
    let metadata = String::from_utf8(buffer.clone()).unwrap_or_default();

    let sensor: Sensor = match serde_json::from_str(&metadata.trim()) {
        Ok(sensor) => sensor,
        Err(e) => {
            eprintln!("Unable to deserialize metadata, connection closed. Err = {}", e);
            return Err(Error::new(ErrorKind::InvalidData, "Invalid sensor data"));
        }
    };

    println!("Sensor connected - Id = {}, Name = {}", sensor.id, sensor.name);
    buffer.clear();
    loop {
        let bytes_read = reader.read_until(b'\n', &mut buffer).await?;
        if bytes_read == 0 {
            println!("Sensor: {} - TCP Connection closed", sensor.name);
            break;
        }

        let message = String::from_utf8(buffer.clone()).unwrap_or_default();
        println!("Sensor: {} - Data: {}", sensor.name, message);
        buffer.clear();
    }

    Ok(())
}

async fn http_handler() -> HttpResponse {
    println!("Http endpoint called");
    HttpResponse::Ok().body("Sensor data coming soon...")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let tcp_server = tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                if let Err(e) = tcp_handler(stream).await {
                    eprintln!("Failed to handle connection: {}", e);
                }
            });
        }
    });

    let http_server = HttpServer::new(|| {
        App::new().route("/", web::get().to(http_handler))
    })
        .bind("127.0.0.1:8080")?
        .run();

    println!("Controller running on 127.0.0.1:3000 (TCP) and 127.0.0.1:8080 (HTTP)");

    tokio::select! {
        tcp_result = tcp_server => {
              if let Err(e) = tcp_result {
                eprintln!("Unable to start TCP Server. Err = {}", e);
                return Err(Error::new(ErrorKind::Other, "TCP Server Error"));
            }
        }

        http_result = http_server => {
            if let Err(e) = http_result {
                eprintln!("Unable to start HTTP Server. Err = {}", e);
                return Err(Error::new(ErrorKind::Other, "HTTP Server Error"));
            }
        }

        _ = signal::ctrl_c() => {
            println!("Controller shutting down...")
        }
    }


    Ok(())
}
