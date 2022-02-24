use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use http_simple::JobFn;
use http_simple::ThreadPool;

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}

fn main() {
    let mut poll = ThreadPool::<JobFn>::new(10);
    poll.run();

    let addr = "localhost:2727";
    let listener = TcpListener::bind(addr).unwrap();
    println!("Server is listenning on {}", addr);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let total_request = poll.execute(Box::new(|| handle_connection(stream)));
        if total_request == 1000 {
            return;
        }
    }
}
