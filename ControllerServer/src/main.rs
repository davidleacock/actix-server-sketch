use std::net::TcpListener;
use actix_server::Server;
use actix_service::fn_service;
use tokio::io::AsyncReadExt;


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

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    println!("Controller running on 127.0.0.1:3000");

    Server::build()
        .listen("tcp_server", listener, || fn_service(tcp_handler))
        .expect("Server failed to run")
        .run()
        .await
}
