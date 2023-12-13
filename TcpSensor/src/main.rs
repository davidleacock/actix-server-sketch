use std::io::{Write, Error};
use std::net::TcpStream;
use std::thread;
use std::env;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use serde_json;
use rand::Rng;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct Sensor {
    id: Uuid,
    name: String
}

fn main() -> Result<(), Error> {
    let mut rng = rand::thread_rng();
    let sensor_id = Uuid::new_v4();
    let sensor_name = env::var("NAME").expect("NAME not provided.");
    let controller_address = env::var("CTRL_ADDR").expect("CTRL_ADDR not provided.");

    let sensor = Sensor {
        id: sensor_id,
        name: sensor_name,
    };

    let serialized_sensor = serde_json::to_string(&sensor).expect("Failed to serialize Sensor.");

    println!("{}", &serialized_sensor);

    let mut stream = TcpStream::connect(&controller_address).expect("Unable to connect to Controller.");

    println!("{:?} started and connected to controller at {}." ,&sensor, &controller_address);

    writeln!(stream, "{}\n", serialized_sensor)?;

    loop {
        let data = rng.gen_range(0..100);

        println!("Current sensor: {}", data);

        if let Err(e) = writeln!(stream, "{}", data) {
            eprintln!("Failed to write to stream: {} - shutting down", e);
            break;
        }

        thread::sleep(Duration::from_secs(2))
    }
    Ok(())
}
