use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::Read;
use dotenv::dotenv;
use std::env;

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).expect("Error: Failed to read from client.");
    
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
fn main() {
    dotenv().ok();

    let port: String = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
 
    let tcp_addr: String = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(tcp_addr).expect("Error: Failed to bind to address.");

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}
