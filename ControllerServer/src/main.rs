use std::net::TcpListener;
use actix_server::Server;
use actix_service::fn_service;
use tokio::io::AsyncReadExt;
use actix_web::{web, App, HttpServer, HttpResponse};
use futures::try_join;

async fn tcp_handler(mut stream: tokio::net::TcpStream) -> std::io::Result<()> {
    let mut buffer = vec![0; 1024];

    loop {
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            println!("TCP Connection closed");
            break;
        }

        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Sensor Data: {}", message);
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
