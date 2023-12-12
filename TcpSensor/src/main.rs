use std::io::{Write, Error};
use std::net::TcpStream;
use std::thread;
use std::env;
use std::time::Duration;

use rand::Rng;

fn main() -> Result<(), Error> {
    let controller_address = env::var("CTRL_ADDR").expect("CTRL_ADDR not provided.");
    let mut stream = TcpStream::connect(&controller_address).expect("Unable to connect to Controller");

    println!("Sensor started and connected to controller at {}...", &controller_address);

    let mut rng = rand::thread_rng();

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
