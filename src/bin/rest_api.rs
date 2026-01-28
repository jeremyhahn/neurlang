//! Simple REST API Server for Neurlang
//!
//! A minimal HTTP server that demonstrates Neurlang's I/O capabilities.
//! Stores state in a local file (state.db).
//!
//! Endpoints:
//! - GET  /value  - Fetch the current value
//! - POST /value  - Set a new value (body is the value)
//! - PUT  /value  - Set a new value (body is the value)
//! - DELETE /value - Reset to default "hello world"

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

const STATE_FILE: &str = "state.db";
const DEFAULT_VALUE: &str = "hello world";
const BIND_ADDR: &str = "127.0.0.1:8080";

fn main() {
    // Initialize state file if it doesn't exist
    if !Path::new(STATE_FILE).exists() {
        fs::write(STATE_FILE, DEFAULT_VALUE).expect("Failed to create state file");
    }

    println!("Neurlang REST API Server");
    println!("========================");
    println!("Listening on http://{}", BIND_ADDR);
    println!();
    println!("Endpoints:");
    println!("  GET    /value  - Fetch the current value");
    println!("  POST   /value  - Set a new value");
    println!("  PUT    /value  - Set a new value");
    println!("  DELETE /value  - Reset to default \"hello world\"");
    println!();
    println!("State stored in: {}", STATE_FILE);
    println!();

    let listener = TcpListener::bind(BIND_ADDR).expect("Failed to bind");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut reader = BufReader::new(&stream);

    // Read the request line
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        send_response(&mut stream, 400, "Bad Request", "Invalid request line");
        return;
    }

    let method = parts[0];
    let path = parts[1];

    // Read headers to get Content-Length
    let mut content_length: usize = 0;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() {
            break;
        }
        let header = header.trim();
        if header.is_empty() {
            break;
        }
        if header.to_lowercase().starts_with("content-length:") {
            if let Some(len_str) = header.split(':').nth(1) {
                content_length = len_str.trim().parse().unwrap_or(0);
            }
        }
    }

    // Read body if present
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        let _ = reader.read_exact(&mut body);
    }
    let body_str = String::from_utf8_lossy(&body);

    // Route the request
    match (method, path) {
        ("GET", "/value") | ("GET", "/") => handle_get(&mut stream),
        ("POST", "/value") | ("POST", "/") => handle_set(&mut stream, &body_str),
        ("PUT", "/value") | ("PUT", "/") => handle_set(&mut stream, &body_str),
        ("DELETE", "/value") | ("DELETE", "/") => handle_delete(&mut stream),
        ("GET", "/health") => send_response(&mut stream, 200, "OK", "healthy"),
        _ => send_response(&mut stream, 404, "Not Found", "Endpoint not found"),
    }
}

fn handle_get(stream: &mut TcpStream) {
    match fs::read_to_string(STATE_FILE) {
        Ok(value) => {
            println!("[GET] Value: {}", value.trim());
            send_json_response(stream, 200, "OK", &value);
        }
        Err(e) => {
            eprintln!("[GET] Error: {}", e);
            send_response(stream, 500, "Internal Server Error", "Failed to read state");
        }
    }
}

fn handle_set(stream: &mut TcpStream, value: &str) {
    let value = value.trim();
    if value.is_empty() {
        send_response(stream, 400, "Bad Request", "Value cannot be empty");
        return;
    }

    match fs::write(STATE_FILE, value) {
        Ok(_) => {
            println!("[SET] Value: {}", value);
            send_json_response(stream, 200, "OK", value);
        }
        Err(e) => {
            eprintln!("[SET] Error: {}", e);
            send_response(
                stream,
                500,
                "Internal Server Error",
                "Failed to write state",
            );
        }
    }
}

fn handle_delete(stream: &mut TcpStream) {
    match fs::write(STATE_FILE, DEFAULT_VALUE) {
        Ok(_) => {
            println!("[DELETE] Reset to default: {}", DEFAULT_VALUE);
            send_json_response(stream, 200, "OK", DEFAULT_VALUE);
        }
        Err(e) => {
            eprintln!("[DELETE] Error: {}", e);
            send_response(
                stream,
                500,
                "Internal Server Error",
                "Failed to reset state",
            );
        }
    }
}

fn send_response(stream: &mut TcpStream, status: u16, status_text: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: text/plain\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, PUT, DELETE\r\n\
         \r\n\
         {}",
        status,
        status_text,
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn send_json_response(stream: &mut TcpStream, status: u16, status_text: &str, value: &str) {
    let json_body = format!("{{\"value\":\"{}\"}}", value.replace('\"', "\\\""));
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, PUT, DELETE\r\n\
         \r\n\
         {}",
        status,
        status_text,
        json_body.len(),
        json_body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}
