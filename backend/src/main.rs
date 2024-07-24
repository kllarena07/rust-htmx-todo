use std::io::{Write, Error};
use std::net::{TcpListener, TcpStream};
use std::io::prelude::Read;
use std::fs;

fn li_elem(task_id: usize, data: &str) -> Result<String, Error> {
    let formatted_task_id = format!("\"{}\"", task_id);

    let base_element = fs::read_to_string("../frontend/components/li-elem.html")?;
    let with_task_id = base_element.replace("{task_id}", &formatted_task_id);
    let with_data = with_task_id.replace("{data}", data);

    Ok(with_data)
}

fn build_list(tasks: &mut Vec<String>) -> Result<String, Error> {
    let mut items_html = String::from("");

    for i in 0..tasks.len() {
        items_html += &li_elem(i, &tasks[i])?;
    }

    let replacement_html = format!("<ul>{}</ul>", items_html);

    Ok(replacement_html)
}

fn handle_home(tasks: &mut Vec<String>) -> Result<String, Error> {
    let list_html = build_list(tasks)?;

    let base_html = fs::read_to_string("../frontend/index.html")?;
    let with_replaced = base_html.replace("|--LIST PLACEHOLDER--|", &list_html);

    Ok(with_replaced)
}

fn handle_connection(mut stream: TcpStream, tasks: &mut Vec<String>) {
    let mut buffer: [u8; 1024] = [0; 1024];

    stream.read(&mut buffer).expect("Error: Failed to read from client.");

    let request = String::from_utf8_lossy(&buffer[..]);

    println!("{}", request);

    let home = b"GET / HTTP/1.1\r\n";
    let create = b"POST /create HTTP/1.1\r\n";
    let delete = b"DELETE /delete HTTP/1.1\r\n";

    let (status_line, content) =
        if buffer.starts_with(home) {
            match handle_home(tasks) {
                Ok(html) => {
                    ("HTTP/1.1 200 OK", html)
                },
                Err(_) => {
                    ("HTTP/1.1 500 INTERNAL SERVER ERROR", String::from("Error building home page."))
                }
            }
        } else if buffer.starts_with(create) {
            let not_found_html: String = fs::read_to_string("../frontend/404.html").unwrap_or_else(|err| {
                eprintln!("Error finding 404 HTML, using default value. Error: {}", err);
                "404 Page Not Found".to_string()
            });

            ("HTTP/1.1 404 NOT FOUND", not_found_html)
        } else if buffer.starts_with(delete) {
            let not_found_html: String = fs::read_to_string("../frontend/404.html").unwrap_or_else(|err| {
                eprintln!("Error finding 404 HTML, using default value. Error: {}", err);
                "404 Page Not Found".to_string()
            });

            ("HTTP/1.1 404 NOT FOUND", not_found_html)
        } else {
            let not_found_html: String = fs::read_to_string("../frontend/404.html").unwrap_or_else(|err| {
                eprintln!("Error finding 404 HTML, using default value. Error: {}", err);
                "404 Page Not Found".to_string()
            });

            ("HTTP/1.1 404 NOT FOUND", not_found_html)
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
    let tcp_addr = "127.0.0.1:3000";

    let listener = TcpListener::bind(tcp_addr).expect("Error: Failed to bind to address.");
    println!("Server listening on {}", tcp_addr);

    let mut tasks: Vec<String> = vec![];

    // single-threaded HTTP server since the current workload + desired scale doesn't require multithreading
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream, &mut tasks);
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }

    }
}
