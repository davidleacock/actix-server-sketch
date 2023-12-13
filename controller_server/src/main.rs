use std::io::{Error, ErrorKind};
use std::net::TcpListener;

use actix_server::Server;
use actix_service::fn_service;
use actix_web::{App, HttpResponse, HttpServer, web};
use futures::try_join;
use serde::Deserialize;
use serde_json;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
struct Sensor {
    id: Uuid,
    name: String,
}

type SensorMap = Arc<Mutex<HashMap<Uuid, Sensor>>>;

async fn tcp_handler(mut stream: TcpStream, sensors: SensorMap) -> std::io::Result<()> {
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

    loop {
        let bytes_read = reader.read_until(b'\n', &mut buffer).await?;
        if bytes_read == 0 {
            println!("TCP Connection closed");
            break;
        }

        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Sensor Data: {}", message);
        buffer.clear();
    }

    Ok(())
}

async fn http_handler() -> HttpResponse {
    println!("Http endpoint called");
    HttpResponse::Ok().body("Sensor data coming soon...")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();

    let tcp_server = Server::build()
        .listen("tcp_server", listener, || fn_service(tcp_handler))
        .expect("Server failed to run")
        .run();

    let http_server = HttpServer::new(|| {
        App::new().route("/", web::get().to(http_handler))
    })
        .bind("127.0.0.1:8080")?
        .run();

    println!("Controller running on 127.0.0.1:3000 (TCP) and 127.0.0.1:8080 (HTTP)");

    try_join!(tcp_server, http_server)?;

    Ok(())
}
