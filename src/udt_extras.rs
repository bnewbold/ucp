
extern crate udt;

use std::str::{self, FromStr};
use udt::{UdtSocket};
use std::io;
use std::io::{Read, Write, ErrorKind};

pub struct UdtStream {
    socket: UdtSocket,
}

impl UdtStream {

    pub fn new(socket: UdtSocket) -> UdtStream {
        UdtStream { socket: socket }

    }
}

impl Read for UdtStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let buf_len = buf.len();
        match self.socket.recv(buf, buf_len) {
            Ok(len) => Ok(len as usize),
            Err(err) => Err(io::Error::new(
                ErrorKind::Other, err.err_msg)),
        }
    }
}

impl Write for UdtStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.socket.send(buf) {
            Ok(len) => Ok(len as usize),
            Err(err) => Err(io::Error::new(
                ErrorKind::Other, err.err_msg)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
