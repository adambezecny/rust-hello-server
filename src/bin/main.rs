use ctrlc;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::fs;
use std::thread;
use std::time::Duration;
use hello::ThreadPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
// use std::io;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    //  https://stackoverflow.com/questions/56692961/graceful-exit-tcplistener-incoming
    // listener.set_nonblocking(true).expect("Cannot set non-blocking");
    let pool = ThreadPool::new(4);
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("Shutting down...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");    
    
    // non-blocking polling not working properly for now
    /* for stream in listener.incoming() {

        let still_running =  running.load(Ordering::SeqCst);
        if still_running == false {
            break;
        }
        
        match stream {
            Ok(s) => {
                pool.execute(|| {
                    handle_connection(s);
                });
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                 println!("WouldBlock error: {}", e);
                thread::sleep(Duration::from_secs(1));
                //
                // would throw A non-blocking socket operation could not be completed immediately. (os error 10035)
                // see https://github.com/tokio-rs/mio/issues/727
                // this is normal for non blocking socket operations when no data is available
                // let's just continue
                continue;
            }
            Err(e) => panic!("encountered IO error: {}", e),
        }        
    } */

    for stream in listener.incoming() {
        // this will shutdown only after accepting new connection, incoming() call block until new connection
        // working this around is pretty complex for now :)
        let still_running =  running.load(Ordering::SeqCst);
        if still_running == false {
            break;
        }

        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }    
    
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/html\r\n\r\n", "404.html")
    };    
    
    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{}{}", status_line, contents);
    
    println!("sending response:\r\n\r\n{}\r\n", response);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();    
}