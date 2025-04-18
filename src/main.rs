use async_std::io;
use async_std::net::TcpListener;
use async_std::prelude::*;



#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();
    let mut incoming = listener.incoming();
    use async_std::task;

    while let Some(stream) = incoming.next().await {
        match stream {
            Ok(mut stream) => {
                task::spawn(async move {
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
                            if *path == "" {
                                response = String::from("HTTP/1.1 200 OK\r\n\r\n");
                            } else if *path == "echo" {
                                if let Some(echo_str) = path_part.get(2) {
                                    response = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                        echo_str.len(),
                                        echo_str
                                    );
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
                                    response = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                        user_agent.len(),
                                        user_agent
                                    );
                                }
                            } else if *path=="files" {
                                if let Some(echo_str) = path_part.get(2) {
                                    let file_path = format!("tmp/{}", echo_str);
                                    let file_content = std::fs::read_to_string(file_path);
                                    match file_content {
                                        Ok(content) => {
                                            response = format!(
                                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                                content.len(),
                                                content
                                            );
                                        }
                                        Err(_) => {
                                            response = String::from("HTTP/1.1 404 Not Found\r\n\r\n");
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
                });
            }
            Err(e) => {
                eprintln!("Erreur de connexion: {}", e);
            }
        }
    }
}


