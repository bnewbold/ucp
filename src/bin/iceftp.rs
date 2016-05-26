
extern crate utp;

use utp::UtpSocket;

fn main() {
    // Bind to port 3540
    let addr = "127.0.0.1:3540";
    let mut socket = UtpSocket::connect(addr).expect("Error connecting to remote peer");

    // Send a string
    socket.send_to("Hi there!".as_bytes()).expect("Write failed");

    // Close the socket
    socket.close().expect("Error closing connection");
}
