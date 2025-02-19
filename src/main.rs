use rust_mini_web_server::ThreadPool;
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;

    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        match stream {
            Ok(stream) => pool.execute(|| {
                if let Err(e) = handle_connections(stream) {
                    eprintln!("Error handling connection: {}", e);
                }
            }),
            Err(e) => eprintln!("Error handling connection: {}", e),
        }
    }

    println!("Shutting down.");
    Ok(())
}

fn handle_connections(mut stream: TcpStream) -> std::io::Result<()> {
    let buf_reader = BufReader::new(&mut stream);

    let request_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => {
            eprintln!("Failed to read request line: {}", e);
            return Err(e);
        }
        None => {
            eprint!("Connection closed before reading request.");
            return Ok(());
        }
    };

    let (status_line, filename) = match request_line.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename)?;
    let response = format!(
        "{status_line}\r\n\
        Content-Length: {}\r\n\r\n\
        {contents}",
        contents.len()
    );

    stream.write_all(response.as_bytes()).unwrap();
    Ok(())
}
