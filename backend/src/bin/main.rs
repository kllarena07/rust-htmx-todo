use std::io::{BufRead, BufReader, Result};
use std::net::{TcpListener, TcpStream};
use std::io::prelude::Read;
use dotenv::dotenv;
use std::{env, fs, fs::OpenOptions, fs::File};
use backend::ThreadPool;
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
            <button data-todo-id=\"{}\" class=\"del-btn\">
            {}
            </button>
            </li>", i, db_data[i]
        );

        items_html += &list_item;
    }

    let replacement_html: String = format!("<ul>{}</ul>", items_html);

    replacement_html
}

fn build_all_todo_html() -> String {
    let replacement_html = build_list_elem();
    let base_html: String = fs::read_to_string("../frontend/index.html").unwrap();
    let with_replaced: String = base_html.replace("|--LIST PLACEHOLDER--|", &replacement_html);

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

fn handle_connection(mut stream: TcpStream) {
    let mut buffer: [u8; 1024] = [0; 1024];

    stream.read(&mut buffer).expect("Error: Failed to read from client.");

    let request = String::from_utf8_lossy(&buffer[..]);

    println!("{}", request);

    let home = b"GET / HTTP/1.1\r\n";
    let create = b"POST /create HTTP/1.1\r\n";
    let delete = b"DELETE /delete HTTP/1.1\r\n";

    let (status_line, content) =
        if buffer.starts_with(home) {
            let home_page_html = build_all_todo_html();

            ("HTTP/1.1 200 OK", home_page_html)
        } else if buffer.starts_with(create) {
            let field_data = extract_field_data(&request, "todo");

            let mut db_file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open("db.txt")
                        .expect("Failed to open or create file.");

            let new_entry = format!("\n{}", field_data);

            db_file.write_all(new_entry.as_bytes()).expect("Error updating db.");

            let rebuilt_element = build_list_elem();

            ("HTTP/1.1 200 OK", rebuilt_element)
        } else if buffer.starts_with(delete) {
            let id_field_data: String = extract_field_data(&request, "todo-id");
            let todo_id = id_field_data.parse::<usize>().unwrap();

            let db_file_data: String = fs::read_to_string("db.txt").unwrap();
            let mut todos: Vec<String> = db_file_data.split('\n').map(|s| s.to_string()).collect();

            todos.remove(todo_id);

            let mut db_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("db.txt")
            .expect("Failed to open or create file.");

            let mut new_data = String::new();

            for i in 0..todos.len() {
                new_data += todos[i].as_str();

                if i != todos.len() - 1 {
                    new_data += "\n";
                }
            }
            
            db_file.write_all(new_data.as_bytes()).unwrap();

            let rebuilt_element = build_list_elem();

            ("HTTP/1.1 200 OK", rebuilt_element)
        } else {
            let not_found_html: String = fs::read_to_string("../frontend/404.html").unwrap();

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
    dotenv().ok();

    let port: String = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
 
    let tcp_addr: String = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(&tcp_addr).expect("Error: Failed to bind to address.");
    println!("Server listening on {}", &tcp_addr);

    let pool: ThreadPool = ThreadPool::new(4);

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
