
extern crate utp;

use std::str;
use std::env;
use std::process::exit;
use getopts::Options;
use utp::{UtpSocket, UtpListener};

pub fn run_client(host: &str, local_file: &str, remote_file: &str, is_recv: bool) {
    println!("host: {}", host);
    println!("local_file: {}", local_file);
    println!("remote_file: {}", remote_file);
    println!("is_recv: {}", is_recv);
}

