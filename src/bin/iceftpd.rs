
extern crate utp;

use utp::{UtpSocket, UtpListener};
use std::str;

fn main() {
    // Connect to an hypothetical local server running on port 3540
    let addr = "127.0.0.1:3540";
    // Accept connection from anybody
    let listener = UtpListener::bind(addr).expect("Error binding to local port");

    for connection in listener.incoming() {

        let (mut socket, _src) = connection.unwrap();
        println!("Got connection from {}", socket.peer_addr().unwrap());

        loop {

            let mut buf = [0; 1000];
            let (amt, _src) = socket.recv_from(&mut buf).ok().unwrap();
            if amt <= 0 {
                break;
            }
            let buf = &buf[..amt];
            let s = str::from_utf8(buf).unwrap();
            println!("\tgot: {}", s);

        }

    }

}
