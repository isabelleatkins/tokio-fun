use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{io, thread};

use std::io::{BufRead, BufReader, Error, Read, Write};
use std::str;

// Handles a single client
fn handle_client(mut stream: TcpStream) -> Result<(), Error> {
    println!("Incoming connection from: {}", stream.peer_addr()?);
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            return Ok(());
        }
        //stream.write(&buf[..bytes_read])?;
        stream.write("replying".as_bytes())?;
    }
}

fn main() {
    println!("start");
    // Server code - spawn it since we're in a synchronous runtime and otherwise this will block
    let (sender, receiver) = std::sync::mpsc::channel::<String>();
    thread::spawn(|| {
        let listener = TcpListener::bind("0.0.0.0:8889").expect("Could not bind");
        for stream in listener.incoming() {
            match stream {
                Err(e) => {
                    eprintln!("failed: {}", e)
                }
                Ok(stream) => {
                    thread::spawn(move || {
                        handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                    });
                }
            }
        }
    });
    //This sleep is to allow the server to start before the client, should improve obviously
    thread::sleep(Duration::from_secs(1));
    println!("before client code");
    // Client code
    let mut stream = TcpStream::connect("127.0.0.1:8889").expect("Could not connect to server");
    println!("do we get here");
    loop {
        let mut input = "jokes".to_string();
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read from stdin");
        stream
            .write(input.as_bytes())
            .expect("Failed to write to server");

        let mut reader = BufReader::new(&stream);

        reader
            .read_until(b'\n', &mut buffer)
            .expect("Could not read into buffer");
        print!(
            "{}",
            str::from_utf8(&buffer).expect("Could not write buffer as string")
        );
    }
}

fn run_server() {
    let listener = TcpListener::bind("0.0.0.0:8888").expect("Could not bind");
    for stream in listener.incoming() {
        match stream {
            Err(e) => {
                eprintln!("failed: {}", e)
            }
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
        }
    }
}
