use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    //
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => { 
                // Création d'un buffer pour stocker les données reçues
                let mut buffer = [0; 1024];
                
                // Lecture des données depuis le stream
                match stream.read(&mut buffer) {
                    Ok(_) => {
                        println!("Request: {}", String::from_utf8_lossy(&buffer));

                        //Get request path
                        let request = String::from_utf8_lossy(&buffer);
                        let request_lines: Vec<&str> = request.split("\r\n").collect();
                        let request_line = request_lines[0];
                        let request_parts: Vec<&str> = request_line.split_whitespace().collect();
                        let path = request_parts[1];

                        // Logique de traitement de la requête
                        let mut response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
                        if path == "/" {
                            response = "HTTP/1.1 200 OK\r\n\r\n";
                        }
                        
                        // Préparation et envoi de la réponse
                        stream.write_all(response.as_bytes()).unwrap();

                        
                        // Assurez-vous que toutes les données sont envoyées avant de passer à la prochaine itération
                        stream.flush().unwrap();
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
