use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{web, App, Error as ActixError, HttpResponse, HttpServer};
use futures::stream;
use futures::stream::Stream;
use libmdns::Responder;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::time;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Sensor {
    id: Uuid,
    name: String,
    value: u32,
}

type SensorState = Arc<Mutex<HashMap<Uuid, Sensor>>>;

async fn tcp_handler(mut stream: TcpStream, sensor_state: SensorState) -> std::io::Result<()> {
    let (reader, _writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut buffer = Vec::new();

    reader.read_until(b'\n', &mut buffer).await?;
    let metadata = String::from_utf8(buffer.clone()).unwrap_or_default();

    let sensor: Sensor = match serde_json::from_str(metadata.trim()) {
        Ok(sensor) => sensor,
        Err(e) => {
            eprintln!(
                "Unable to deserialize metadata, connection closed. Err = {}",
                e
            );
            return Err(Error::new(ErrorKind::InvalidData, "Invalid sensor data"));
        }
    };

    println!("Sensor: {} connected - id:{}", sensor.name, sensor.id);
    buffer.clear();
    loop {
        let bytes_read = reader.read_until(b'\n', &mut buffer).await?;
        if bytes_read == 0 {
            println!(
                "Sensor: {} - TCP Connection closed - Removing from state",
                sensor.name
            );
            let mut data = sensor_state.lock().unwrap();
            data.remove(&sensor.id);
            break;
        }

        let sensor_value = String::from_utf8(buffer.clone()).unwrap_or_default();
        println!("Sensor: {} - Data: {}", sensor.name, sensor_value);

        let mut sensors = sensor_state.lock().unwrap();

        if let Some(sensor) = sensors.get_mut(&sensor.id) {
            sensor.value = sensor_value.trim().parse::<u32>().unwrap_or(99999);
        } else {
            sensors.insert(sensor.id, sensor.clone());
        }

        buffer.clear();
    }
    Ok(())
}

fn stream_sensor_state(
    sensor_state: SensorState,
) -> impl Stream<Item = Result<web::Bytes, ActixError>> {
    stream::unfold(sensor_state, |state| async {
        time::sleep(Duration::from_secs(3)).await;

        let json = {
            let data = state.lock().unwrap();
            serde_json::to_string(&*data).unwrap()
        };

        let stream_data = format!("data: {}\n\n", json);

        Some((Ok(web::Bytes::from(stream_data)), state))
    })
}

async fn http_handler(sensor_state: SensorState) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(stream_sensor_state(sensor_state))
}

async fn tcp_server(sensor_state: SensorState) -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let data = sensor_state.clone();
        tokio::spawn(async move {
            println!("Connection discovered");
            if let Err(e) = tcp_handler(stream, data).await {
                eprintln!("Failed to handle connection: {}", e);
            }
        });
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut builder = env_logger::Builder::new();
    builder.parse_filters("libmdns=debug");
    builder.init();

    let sensor_state: SensorState = Arc::new(Mutex::new(HashMap::new()));

    // I need to keep registering the service otherwise the tcp sensors don't seem to connect
    tokio::task::spawn(async {
        loop {
            let responder = Responder::new().unwrap();
            let _service = responder.register(
                "_controller-server._tcp".to_owned(),
                "ControllerServer".to_owned(),
                3000, // TODO this could be a configurable param
                &["path=/"],
            );

            // Keep the service registered for a certain duration
            tokio::time::sleep(Duration::from_secs(5)).await;
            // The loop will then re-register the service
        }
    });

    let tcp_server = tokio::spawn(tcp_server(sensor_state.clone()));

    let http_server = HttpServer::new(move || {
        let state_clone = sensor_state.clone();
        App::new().wrap(Cors::permissive()).route(
            "/display",
            web::get().to(move || {
                let state = state_clone.clone();
                async move { http_handler(state).await }
            }),
        )
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
            println!("Controller shutting down....")
        }
    }

    Ok(())
}
