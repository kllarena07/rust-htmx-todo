use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::io::prelude::Read;
use dotenv::dotenv;
use std::{env, fs};
use std::thread;

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).expect("Error: Failed to read from client.");
    
    let get = b"GET / HTTP/1.1\r\n";

    if buffer.starts_with(get) {
        let contents = fs::read_to_string("../frontend/index.html").unwrap();
    
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else {
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let contents = fs::read_to_string("../frontend/404.html").unwrap();
    
        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            contents.len(),
            contents
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
fn main() {
    dotenv().ok();

    let port: String = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
 
    let tcp_addr: String = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(&tcp_addr).expect("Error: Failed to bind to address.");
    println!("Server listening on {}", &tcp_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_connection(stream));
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }

    }
}
