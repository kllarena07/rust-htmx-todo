use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::io::prelude::Read;
use dotenv::dotenv;
use std::{env, fs};
use backend::ThreadPool;

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).expect("Error: Failed to read from client.");

    let request = String::from_utf8_lossy(&buffer[..]);

    println!("{}", request);

    let read_data: Vec<String> = match fs::read_to_string("db.txt") {
        Ok(data) => {
            let data_array = data.split('\n').map(|s| s.to_string()).collect();
            data_array
        },
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let home = b"GET / HTTP/1.1\r\n";

    if buffer.starts_with(home) {
        let mut html = String::from(
            "
            <!DOCTYPE html>\n
            <html lang=\"en\">\n
            <head>\n
                <meta charset=\"utf-8\">\n
                <title>HTMX/Rust Todo</title>\n
            </head>\n
            <body>\n
                <h1>Todos:</h1>\n
                <ul>
            "
        );

        for data in read_data {
            let todo = format!(
                "
                <li>\n
                    {}\n
                </li>\n
                ", data
            );

            html += &todo;
        }

        html +=
        "
            </ul>\n
            </body>\n
            </html>
        ";

        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            "HTTP/1.1 200 OK",
            html.len(),
            html
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

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }

    }
}
