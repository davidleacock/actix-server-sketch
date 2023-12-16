use std::env;
use std::io::{Error, ErrorKind, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;

use futures::stream::StreamExt;
use mdns::discover;
use mdns::RecordKind;
use pin_utils::pin_mut;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct Sensor {
    id: Uuid,
    name: String,
    value: u32
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut rng = rand::thread_rng();
    let sensor_id = Uuid::new_v4();
    let sensor_name = env::var("NAME").unwrap_or(String::from("no_name"));

    let sensor = Sensor {
        id: sensor_id,
        name: sensor_name.clone(),
        value: 0
    };

    let serialized_sensor = serde_json::to_string(&sensor).expect("Failed to serialize Sensor.");

    println!("{}", &serialized_sensor);

    // TODO Does this only work with ".local" and the Controller doesnt have that?
        let controller_service = "_controller-server._tcp.local";
    let mut controller_address: Option<SocketAddr> = None;

    loop {
        println!("Starting loop...");
        let stream = match discover::all(controller_service, Duration::from_secs(60)) {
            Ok(discover) => {
                println!("Discovery found listening...");
                discover.listen()
            }
            Err(e) => {
                eprintln!("error with discovery: {}", e);
                return Err(Error::new(ErrorKind::Other, "Discover error"));
            }
        };

        pin_mut!(stream); // Pin the stream in place while we read it
        println!("Waiting on response in stream");

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for record in response.records() {
                        match &record.kind {
                            RecordKind::A(ipv4) => {
                                println!("Discovered ipv4 address: {}", &ipv4);
                                controller_address = Some(SocketAddr::new(IpAddr::V4(*ipv4), response.port().unwrap()));
                                break;
                            }
                            RecordKind::AAAA(ipv6) => {
                                println!("Discovered ipv6 address: {}", &ipv6);
                                controller_address = Some(SocketAddr::new(IpAddr::V6(*ipv6), response.port().unwrap()));
                                break;
                            }

                            RecordKind::SRV { priority, weight, port, target } => {
                                println!("SRV priority: {} weight: {}, port: {}, target: {}", priority, weight, port, target);
                            }

                            _ => {}
                        }
                    }

                    if controller_address.is_some() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Stream error: {:?}", e);
                    break;
                }
            }
        }

        if controller_address.is_some() {
            println!("Discovered ControllerServer at {}", controller_address.unwrap());
            break;
        } else {
            println!("ControllerServer not found. Retrying in 5 seconds...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }


    let controller_address = controller_address.expect("Unable to find ControllerServer");
    println!("Discovered ControllerServer at {}", &controller_address);

    let mut stream = TcpStream::connect(controller_address).expect("Unable to connect to Controller.");

    println!("{:?} started and connected to controller at {}.", &sensor, &controller_address);

    writeln!(stream, "{}\n", serialized_sensor)?;

    loop {
        let data = rng.gen_range(0..100);

        println!("Sensor: {} current value: {}", &sensor_name, data);

        if let Err(e) = writeln!(stream, "{}", data) {
            eprintln!("Failed to write to stream: {} - shutting down", e);
            break;
        }

        thread::sleep(Duration::from_secs(2))
    }
    Ok(())
}
