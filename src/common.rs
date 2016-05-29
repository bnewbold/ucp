
extern crate utp;

use std::str;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::exit;
use utp::{UtpSocket};


pub fn send_files(socket: &mut UtpSocket, file_path: &str, recursive: bool) {
    assert!(!recursive);
    let f = File::open(file_path).unwrap();
    unimplemented!();
}

pub fn receive_files(socket: &mut UtpSocket, file_path: &str, recursive: bool) {
    assert!(!recursive);
    let f = File::create(file_path).unwrap();

    //f.set_len();
}
