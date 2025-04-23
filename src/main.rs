use std::env;
use std::path::PathBuf;
use async_std::io::{ReadExt, WriteExt};
use async_std::net::TcpListener;
use async_std::net::TcpStream;

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


            let mut response = String::from("HTTP/1.1 404 Not Found\r\n\r\n");
            let method = request_parts.get(0).unwrap_or(&"GET");
            let mut accept_encoding = String::new();
            for line in request_lines.iter() {
                if line.starts_with("Accept-Encoding:") {
                    // Extract the value after the header name and colon
                    let parts: Vec<_> = line.split(':').collect();
                    if parts.len() > 1 {
                        // Trim whitespace from the value part
                        accept_encoding = parts[1].trim().to_string();
                    }
                    break;
                }
            }

            if *method == "GET" {
                if *path == "" {
                    response = String::from("HTTP/1.1 200 OK\r\n\r\n");
                } else if *path == "echo" {
                    if let Some(echo_str) = path_part.get(2) {
                        // Check if the client accepts gzip encoding
                        if accept_encoding.contains("gzip") {
                            response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n{}",
                                echo_str.len(),
                                echo_str
                            );
                        } else {
                            response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                echo_str.len(),
                                echo_str
                            );
                        }
                    }

                } else if *path == "user-agent" {
                    let mut user_agent_line = String::new();
                    for line in request_lines.iter() {
                        if line.starts_with("User-Agent:") {
                            user_agent_line = line.to_string();
                            break;
                        }
                    }
                    if let Some(user_agent) = user_agent_line.split(": ").nth(1) {
                        if accept_encoding.contains("gzip") {
                            response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n{}",
                                user_agent.len(),
                                user_agent
                            );

                        }else {
                            response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                user_agent.len(),
                                user_agent
                            );
                        }

                    }
                } else if *path == "files" {
                    if let Some(file_name) = path_part.get(2) {
                        let file_path = PathBuf::from(&directory).join(file_name);
                        // Utiliser le fs synchrone pour lire le contenu du fichier
                        // Dans un serveur de production, vous devriez utiliser fs::read_to_string
                        let file_content = std::fs::read_to_string(&file_path);
                        match file_content {
                            Ok(content) => {
                                response = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                                    content.len(),
                                    content
                                );
                            }
                            Err(e) => {
                                eprintln!("Erreur lors de la lecture du fichier {:?}: {}", file_path, e);
                                response = String::from("HTTP/1.1 404 Not Found\r\n\r\n");
                            }
                        }
                    }
                }
            } else if *method == "POST" {
                if *path == "files" {
                    if let Some(file_name) = path_part.get(2) {
                        let file_path = PathBuf::from(&directory).join(file_name);

                        // Extract the body of the POST request
                        let body_start = request.find("\r\n\r\n").map(|pos| pos + 4).unwrap_or(0);
                        let body = &request[body_start..].replace("\x00","");

                        // Write the body to the specified file
                        match std::fs::write(&file_path, body) {
                            Ok(_) => {
                                response = String::from("HTTP/1.1 201 Created\r\n\r\n");
                            }
                            Err(e) => {
                                eprintln!("Erreur lors de l'écriture du fichier {:?}: {}", file_path, e);
                                response = String::from("HTTP/1.1 500 Internal Server Error\r\n\r\n");
                            }
                        }
                    }
                }
            }

            if let Err(e) = stream.write_all(response.as_bytes()).await {
                eprintln!("Erreur d'écriture: {}", e);
            }
            if let Err(e) = stream.flush().await {
                eprintln!("Erreur de flush: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Erreur de lecture: {}", e);
        }
    }
}
