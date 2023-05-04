use std::io::Read;
use std::io::Write;
use std::str;
use std::net::Shutdown;
use std::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::process::Command;
use std::process::Output;
use std::error::Error;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum DataType {
    Shutdown,
    Command(String),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OutputData {
    error: bool,
    stdout: String,
    stderror: String,
}

impl From<Output> for OutputData {
    fn from(value: Output) -> Self {
        OutputData {
            error: !value.status.success(),
            stdout: String::from_utf8_lossy(&value.stdout).to_string(),
            stderror: String::from_utf8_lossy(&value.stderr).to_string(),
        }
    }
}

fn execute_command(command: &str) -> std::io::Result<Output> {
    Command::new("sh").args(["-c", command]).output()
}

fn shutdown() -> std::io::Result<Output> {
    Command::new("shutdown").output()
}

#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {
    
    // Notify the discord bot that the server has been turned on
    let server_ip = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192,168,1,201)), 2139);
    let mut stream = TcpStream::connect(server_ip)?;

    stream
        .set_write_timeout(Some(Duration::new(10, 0)))
        .expect("set_write_timeout call failed");
    stream
        .set_read_timeout(Some(Duration::new(10, 0)))
        .expect("set_read_timeout call failed");

    stream.write_all(b"{\"type_of_event\": \"turn_on\", \"player\": \"Server\", \"content\": \"\"}")?;
    stream.shutdown(Shutdown::Both)?;
    drop(stream);

    let listening_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 2137);
    let listener = TcpListener::bind(listening_address).expect("Failed to create TcpListener");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut response_buffer = String::new();
                    if let Err(e) = stream.read_to_string(&mut response_buffer) {
                        println!("Error {e}")
                    }
                    let decoded: DataType = match ron::from_str(&response_buffer) {
                        Ok(t) => t,
                        Err(e) => {
                            println!("Error {e}");
                            continue;
                        }
                    };
                    let output = match decoded {
                        DataType::Shutdown => OutputData::from(shutdown()?),
                        DataType::Command(s) => OutputData::from(execute_command(&s)?)
                    };
                    if let Err(e) = stream.write_all(ron::to_string(&output)?.as_bytes()) {
                        println!("Error with writing to stream {e}")
                }
                    
                }
                Err(e) => println!("{e}"),
            }
        }
        Ok(())
}
