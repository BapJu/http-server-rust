use std::{env, io};
use std::path::PathBuf;
use async_std::io::{ReadExt, WriteExt};
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.

    println!("Logs from your program will appear here!");

    // Récupérer le répertoire des fichiers depuis les arguments de ligne de commande
    let args: Vec<String> = env::args().collect();
    let mut directory = String::from("/tmp");
    let mut data = String::from("");

    for i in 0..args.len() {
        if args[i] == "--directory" && i + 1 < args.len() {
            directory = args[i + 1].clone();
        } if args[i] == "--data" && i + 2 < args.len() {
            data = args[i + 2].clone();
        }
    }

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let directory_clone = directory.clone();
                let data_clone = data.clone();
                tokio::spawn(async move {
                    handle_connection(stream, directory_clone, data_clone).await;
                });
            }
            Err(e) => {
                eprintln!("Erreur de connexion: {}", e);
            }
        }
    }
}


async fn handle_connection(mut stream: TcpStream, directory: String, data: String) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer).await {
        Ok(_) => {
            let request = String::from_utf8_lossy(&buffer);
            let request_lines: Vec<&str> = request.split("\r\n").collect();
            let request_line = request_lines[0];
            let request_parts: Vec<&str> = request_line.split_whitespace().collect();
            let all_paths = request_parts.get(1).unwrap_or(&"");
            let path_part: Vec<&str> = all_paths.split('/').collect();
            let path = path_part.get(1).unwrap_or(&"");

            let method = request_parts.get(0).unwrap_or(&"GET");
            let mut accept_encoding = String::new();
            for line in request_lines.iter() {
                if line.starts_with("Accept-Encoding:") {
                    let parts: Vec<_> = line.split(':').collect();
                    if parts.len() > 1 {
                        accept_encoding = parts[1].trim().to_string();
                    }
                    break;
                }
            }

            // Default: 404
            let mut response_bytes = b"HTTP/1.1 404 Not Found\r\n\r\n".to_vec();

            if *method == "GET" {
                if *path == "" {
                    response_bytes = b"HTTP/1.1 200 OK\r\n\r\n".to_vec();
                } else if *path == "echo" {
                    if let Some(echo_str) = path_part.get(2) {
                        // Convert to hex
                        let hex_rep = echo_str.bytes()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>();

                        if accept_encoding.contains("gzip") {
                            // Gzip compress
                            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                            encoder.write_all(hex_rep.as_bytes()).unwrap();
                            let compressed = encoder.finish().unwrap();
                            let header = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n",
                                compressed.len()
                            );
                            // Build binary response
                            response_bytes = header.into_bytes();
                            response_bytes.extend_from_slice(&compressed);
                        } else {
                            let body = hex_rep;
                            let header = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                body.len(), body
                            );
                            response_bytes = header.into_bytes();
                        }
                    }
                } else if *path == "user-agent" {
                    let mut user_agent = "";
                    for line in request_lines.iter() {
                        if line.starts_with("User-Agent:") {
                            user_agent = line.split(": ").nth(1).unwrap_or("");
                            break;
                        }
                    }
                    if !user_agent.is_empty() {
                        if accept_encoding.contains("gzip") {
                            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                            encoder.write_all(user_agent.as_bytes()).unwrap();
                            let compressed = encoder.finish().unwrap();
                            let header = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n",
                                compressed.len()
                            );
                            response_bytes = header.into_bytes();
                            response_bytes.extend_from_slice(&compressed);
                        } else {
                            let header = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                user_agent.len(), user_agent
                            );
                            response_bytes = header.into_bytes();
                        }
                    }
                } else if *path == "files" {
                    if let Some(file_name) = path_part.get(2) {
                        let path = PathBuf::from(&directory).join(file_name);
                        let file_content = std::fs::read(&path); // as Vec<u8>

                        match file_content {
                            Ok(content) => {
                                let header = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                                    content.len()
                                );
                                response_bytes = header.into_bytes();
                                response_bytes.extend_from_slice(&content);
                            }
                            Err(e) => {
                                eprintln!("File read error {:?}: {}", path, e);
                                response_bytes = b"HTTP/1.1 404 Not Found\r\n\r\n".to_vec();
                            }
                        }
                    }
                }
            } else if *method == "POST" {
                if *path == "files" {
                    if let Some(file_name) = path_part.get(2) {
                        let file_path = PathBuf::from(&directory).join(file_name);
                        let body_start = request.find("\r\n\r\n").map(|pos| pos + 4).unwrap_or(0);
                        let body = &request[body_start..].replace("\x00", "");
                        match std::fs::write(&file_path, body) {
                            Ok(_) => response_bytes = b"HTTP/1.1 201 Created\r\n\r\n".to_vec(),
                            Err(e) => {
                                eprintln!("Write error {:?}: {}", file_path, e);
                                response_bytes = b"HTTP/1.1 500 Internal Server Error\r\n\r\n".to_vec();
                            }
                        }
                    }
                }
            }

            if let Err(e) = stream.write_all(&response_bytes).await {
                eprintln!("Write error: {}", e);
            }
            if let Err(e) = stream.flush().await {
                eprintln!("Flush error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Read error: {}", e);
        }
    }
}
