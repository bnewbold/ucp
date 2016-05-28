
extern crate utp;

use std::string::String;
use std::env;
use std::process;
use std::process::Command;
use utp::{UtpSocket};

pub fn run_client(host: &str, local_file: &str, remote_file: &str, remote_is_dir: bool, is_recv: bool) {
    println!("\thost: {}", host);
    println!("\tlocal_file: {}", local_file);
    println!("\tremote_file: {}", remote_file);
    println!("\tis_recv: {}", is_recv);

    let mut ssh_cmd = Command::new("ssh");
    ssh_cmd.arg(host)
           .arg("--")
           .arg("ucp")
           .arg("server")
           .arg(if is_recv {"-f"} else {"-t"})
           .arg(remote_file);

    if remote_is_dir {
        ssh_cmd.arg("-d");
    }

    let ssh_output = ssh_cmd.output().expect("couldn't get SSH sub-process output");

    if !ssh_output.status.success() {
        panic!("SSH problem: {}", String::from_utf8_lossy(&ssh_output.stderr));
    }

    let reply = String::from_utf8_lossy(&ssh_output.stdout);
    println!("SSH reply: {}", reply);
    let words: Vec<&str> = reply.split_whitespace().collect();
    if words.len() != 5 || words[0] != "UDP" || words[1] != "CONNECT" {
        panic!("Unexpected data via SSH pipe (TCP)");
    }
    let remote_host = words[2];
    let remote_port = words[3].parse::<u16>().expect("failed to parse remote port number");
    let remote_secret = words[4];

    println!("Got remote details:");
    println!("\tport: {}", remote_port);
    println!("\thost: {}", remote_host);
    println!("\tsecret: {}", remote_secret);

    let mut buf = [0; 2000];
    let mut socket = UtpSocket::connect((remote_host, remote_port)).unwrap();;
    socket.send_to("PING".as_bytes());
    socket.flush();
    let (amt, _src) = socket.recv_from(&mut buf).ok().unwrap();
    let reply = String::from_utf8_lossy(&buf[..amt]);
    println!("Got uTP reply: {}", reply);
    socket.close();
}
