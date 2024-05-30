use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};

use std::io::{BufRead, BufReader, Error, Read, Write};
use std::str;

// This main function is an exercise to demonstrate sharing data across threads in different ways: shared memory and channels. Note this is a synchronous runtime.
// It's also a demonstration of blocking threads, and spawning them to get around that.
fn main() {
    // Set up an Arc Mutex Vec to demonstrate one form of thread communication: shared memory
    let messages = Arc::new(Mutex::new(Vec::new()));
    // Set up a sender/receiver channel which will be used to send data across threads - to demonstrate one form of thread communication: channels
    let (sender, receiver) = std::sync::mpsc::channel();
    // Clone the messages Arc Mutex Vec so we can use it in the client handling threads (which will insert messages into it) and the receiver thread (which will read messages from it)
    let messages1 = Arc::clone(&messages);
    // Server code - spawn it since we're in a synchronous runtime and otherwise this will block
    thread::spawn(move || {
        let listener = TcpListener::bind("0.0.0.0:8889").expect("Could not bind");
        for stream in listener.incoming() {
            match stream {
                Err(e) => {
                    eprintln!("failed: {}", e)
                }
                Ok(stream) => {
                    let sender = sender.clone();
                    let messages = Arc::clone(&messages1);
                    thread::spawn(move || {
                        handle_client(stream, sender, messages)
                            .unwrap_or_else(|error| eprintln!("{:?}", error));
                    });
                }
            }
        }
    });

    // Spawn thread for receiver
    thread::spawn(move || {
        let mut messages_from_channel = Vec::new();
        for message in receiver.iter() {
            messages_from_channel.push(message.clone());
            println!(
                "Received: {}, new list of messages: {:#?}",
                message, messages_from_channel
            );
            println!("here's the mutext: {:#?}", messages.lock().unwrap());
        }
    });

    //This sleep is to allow the server to start before the client, should improve obviously but that's not the goal of this example
    thread::sleep(Duration::from_secs(1));
    println!("before client code");

    // Spin up a client connection to send some data to it - this is just to demonstrate the server/client communication.
    // We don't need to spawn a new thread despite it being a blocking call because we've spawned all other responsibilities to other threads.
    let mut stream = TcpStream::connect("127.0.0.1:8889").expect("Could not connect to server");
    println!("do we get here");
    loop {
        let mut input = String::new();
        let mut buffer: Vec<u8> = Vec::new();
        // This next bit means that someone writing to stdin (ie just typing once they run main into console) will be read in.
        // Alternatively use the nc command to connect to the server and type in the terminal eg "nc 127.0.0.1 8889"
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read from stdin");
        stream
            .write(input.as_bytes())
            .expect("Failed to write to server");
        println!("ding ding");
        let mut reader = BufReader::new(&stream);
        println!("before read_until");
        reader
            .read_until(b'\n', &mut buffer)
            .expect("Could not read into buffer");
        println!(
            "{}",
            str::from_utf8(&buffer).expect("Could not write buffer as string")
        );
    }
}

// Handles a single client
fn handle_client(
    mut stream: TcpStream,
    sender: Sender<String>,
    messages: Arc<Mutex<Vec<String>>>,
) -> Result<(), Error> {
    println!("Incoming connection from: {}", stream.peer_addr()?);
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            return Ok(());
        }
        // Let's send the message to the main thread, to demonstrate our channel sending between threads
        let message = str::from_utf8(&buf[..bytes_read]).unwrap().to_string();
        messages.lock().unwrap().push(message.clone());
        sender.send(message).unwrap();
        //stream.write(&buf[..bytes_read])?;
        stream.write("replying".as_bytes())?;
    }
}
