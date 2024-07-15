use std::io::{BufRead, BufReader, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::io::prelude::Read;
use dotenv::dotenv;
use std::{env, fs, fs::OpenOptions, fs::File};
use std::borrow::Cow;

fn get_db_data() -> Result<Vec<String>> {
    let file = File::open("db.txt")?;
    let reader = BufReader::new(file);
    let data: Vec<String> = reader.lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(data)
}

fn build_list_elem() -> String {
    let db_data = get_db_data().unwrap();

    let mut items_html: String = String::from("");

    for i in 0..db_data.len() {
        let list_item: String = format!(
            "<li>
            <button data-task-id=\"{}\" class=\"del-btn\">
            {}
            </button>
            </li>", i, db_data[i]
        );

        items_html += &list_item;
    }

    let replacement_html: String = format!("<ul>{}</ul>", items_html);

    replacement_html
}

fn build_page_html() -> String {
    let list_html = build_list_elem();
    let base_html: String = fs::read_to_string("../frontend/index.html").unwrap();
    let with_replaced: String = base_html.replace("|--LIST PLACEHOLDER--|", &list_html);

    with_replaced
}

fn extract_field_data(request: &Cow<str>, field_name: &str) -> String {
    let field = format!("Content-Disposition: form-data; name=\"{}\"", field_name);

    let index = match request.find(&field) {
        Some(number) => {
            number
        },
        None => {
            eprintln!("Could not locate Content-Disposition");
            0
        }
    };

    let body_start: &str = &request[index..];
    let body_end = match body_start.find("---") {
        Some(number) => {
            number
        },
        None => {
            eprintln!("Could not locate the end.");
            0
        }
    };

    let body_data: &str = &request[index..(index + body_end)];
    let mut first_field: Vec<String> = body_data.split("\r\n").map(|s| s.to_string()).collect();
    first_field.retain(|s| !s.trim().is_empty());

    let field_data: String = first_field[1].clone();
    field_data
}

fn add_task(task: &str) {
    let mut db_file = OpenOptions::new()
                        .append(true)
                        .open("db.txt")
                        .expect("Failed to open file.");

    // Task should already be formatted with a '\n' in front
    db_file.write_all(task.as_bytes()).expect("Error adding task to db.");
}

fn remove_task(task_id: usize) {
    let mut db_data = get_db_data().unwrap();

    db_data.remove(task_id);

    let mut db_file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .open("db.txt")
    .expect("Failed to open file.");

    let mut new_data = String::new();

    for i in 0..db_data.len() {
        new_data += "\n";
        new_data += db_data[i].as_str();
    }
    
    db_file.write_all(new_data.as_bytes()).expect("Error removing task from db.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer: [u8; 1024] = [0; 1024];

    stream.read(&mut buffer).expect("Error: Failed to read from client.");

    let request = String::from_utf8_lossy(&buffer[..]);

    println!("{}", request);

    let home = b"GET / HTTP/1.1\r\n";
    let create = b"POST /create HTTP/1.1\r\n";
    let delete = b"DELETE /delete HTTP/1.1\r\n";

    let (status_line, content) = match buffer {
        b if b.starts_with(home) => {
            let home_page_html = build_page_html();

            ("HTTP/1.1 200 OK", home_page_html)
        },
        b if b.starts_with(create) => {
            let task_data = extract_field_data(&request, "task");

            let formatted_task = format!("\n{}", task_data);

            add_task(&formatted_task);

            let rebuilt_element = build_list_elem();

            ("HTTP/1.1 200 OK", rebuilt_element)
        },
        b if b.starts_with(delete) => {
            let id_field_data: String = extract_field_data(&request, "task-id");
            let task_id = id_field_data.parse::<usize>().unwrap();

            remove_task(task_id);

            let rebuilt_element = build_list_elem();

            ("HTTP/1.1 200 OK", rebuilt_element)
        }
        _ => {
            let not_found_html: String = fs::read_to_string("../frontend/404.html").unwrap_or_else(|err| {
                eprintln!("Error finding 404 HTML, using default value. Error: {}", err);
                "404 Page Not Found".to_string()
            });

            ("HTTP/1.1 404 NOT FOUND", not_found_html)
        }
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

    // single-threaded HTTP server since the current workload doesn't require multithreading
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }

    }
}
