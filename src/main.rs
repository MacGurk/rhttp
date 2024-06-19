use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{fs, process, thread};
use hello::ThreadPool;

fn main() {
    let listener = match TcpListener::bind("0.0.0.0:7878") {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Application error: {e}");
            process::exit(1);
        }
    };

    let pool = ThreadPool::new(4);

    for stream in  listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}


fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader
        .lines()
        .next()
        .unwrap()
        .unwrap();

    println!("Request: {:#?}", request_line);

    let request_line_parts: Vec<_> = request_line.split(' ').collect();

    let http_version = request_line_parts[2];
    if !http_version.contains("HTTP") {
        let status_line = "HTTP/1.1 400 Bad Request";
        let response = format!(
            "{status_line}\r\n\
            Content-Type: text/plain; chatset=utf-8\r\n\
            Connection: close\r\n\r\n"
        );

        stream.write_all(response.as_bytes()).unwrap();
        return;
    }


    let root_dir = "www";
    let request_path = request_line_parts[1];
    let path = format!("{}{}", root_dir, request_path);

    let contents_result = fs::read_to_string(path);

    let (status_line, contents) = match contents_result {
        Ok(content) => ("HTTP/1.1 200 OK", content),
        _ => ("HTTP/1.1 404 NOT FOUND", format!(
                    "
<!DOCTYPE html>
<html lang=\"en\">
    <head>
        <meta charset=\"utf-8\">
        <title>404 Not Found</title>
    </head>
    <body>
        <h1>Not found!</h1>
        <p>The requested URL {request_path} was not found on this server!</p>
    </body>
</html>"
            )),
        };

    let length = contents.len();

    let response = format!(
        "{status_line}\r\n\
        Content-Length: {length}\r\n\
        Content-Type: text/html; charset=utf-8\r\n\r\n\
        {contents}"
    );

    println!("Response: {:#?}", response);

    stream.write_all(response.as_bytes()).unwrap();
}
