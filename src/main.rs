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
    while let Some(stream) = incoming.next().await {
        match stream {
            Ok(mut stream) => {
                // Création d'un buffer pour stocker les données reçues
                let mut buffer = [0; 1024];

                // Lecture des données depuis le stream
                match stream.read(&mut buffer).await {
                    Ok(_) => {
                        println!("Request: {}", String::from_utf8_lossy(&buffer));

                        //Get request path
                        let request = String::from_utf8_lossy(&buffer);
                        let request_lines: Vec<&str> = request.split("\r\n").collect();
                        let request_line = request_lines[0];
                        let request_parts: Vec<&str> = request_line.split_whitespace().collect();
                        let all_paths = request_parts[1];
                        let path_part : Vec<&str> = all_paths.split("/").collect();
                        let path = path_part[1];


                        let mut response = String::from("HTTP/1.1 404 Not Found\r\n\r\n");
                        if path == "" {
                            response = String::from("HTTP/1.1 200 OK\r\n\r\n");
                        }
                        else if path=="echo" {
                            let echo_str = path_part[2];
                            response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", echo_str.len(), echo_str);

                        }
                        else if path=="user-agent" {
                            //loking for user-agent on request lines (not especilly on the first line)
                            let mut user_agent_line = String::new();
                            for line in request_lines.iter() {
                                if line.starts_with("User-Agent:") {
                                    user_agent_line = line.to_string();
                                    break;
                                }
                            }




                            let user_agent = user_agent_line.split(": ").collect::<Vec<&str>>()[1];
                            println!("User-agent: {}", user_agent);
                            println!("User-agent len : {}", user_agent.len());
                            response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", user_agent.len(), user_agent);
                        }


                        stream.write_all(response.as_bytes()).await.unwrap();



                        stream.flush().await.unwrap();
                    },
                    Err(e) => {
                        println!("Erreur de lecture: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
        
}
    
        

