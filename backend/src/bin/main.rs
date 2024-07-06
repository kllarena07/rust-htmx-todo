use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::io::prelude::Read;
use dotenv::dotenv;
use std::{env, fs};
use backend::ThreadPool;

fn build_all_todo_html() -> String {
    let replacement_html: String = match fs::read_to_string("db.txt") {
        Ok(data) => {
            let data_array: Vec<String> = data.split('\n').map(|s| s.to_string()).collect();

            let mut list_html = String::from("<ul>\n");

            for i in 0..data_array.len() {
                let list_item = format!(
                    "
                    <li data-todo-id=\"{}\">\n
                        {}\n
                    </li>\n
                    ", i, data_array[i]
                );

                list_html += list_item.as_str();
            }

            list_html += "</ul>\n";

            list_html
        },
        Err(e) => {
            eprintln!("{}", e);
            String::from("<p>Error fetching todos</p>")
        }
    };

    let base_html = fs::read_to_string("../frontend/index.html").unwrap();
    let with_replaced = base_html.replace("|--TODOS PLACEHOLDER--|", &replacement_html);

    with_replaced
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).expect("Error: Failed to read from client.");

    let request = String::from_utf8_lossy(&buffer[..]);

    println!("{}", request);

    let home = b"GET / HTTP/1.1\r\n";

    let (status_line, content) =
        if buffer.starts_with(home) {
            let home_page_html = build_all_todo_html();

            ("HTTP/1.1 200 OK", home_page_html)
        } else {
            let unknown_html = fs::read_to_string("../frontend/404.html").unwrap();

            ("HTTP/1.1 404 NOT FOUND", unknown_html)
        };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content.len(),
        content
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
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
