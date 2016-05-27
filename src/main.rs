
extern crate getopts;
extern crate utp;

use std::str;
use std::env;
use std::process::exit;
use getopts::Options;
use utp::{UtpSocket, UtpListener};


fn run_server(path: &str, is_receive: bool) {
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

fn run_client(path: &str, is_receive: bool) {
    // Bind to port 3540
    let addr = "127.0.0.1:3540";
    let mut socket = UtpSocket::connect(addr).expect("Error connecting to remote peer");

    // Send a string
    socket.send_to("Hi there!".as_bytes()).expect("Write failed");

    // Close the socket
    socket.close().expect("Error closing connection");
}

fn send_files(socket: UtpSocket, file_path: &str, recursive: bool) {
    unimplemented!();
}

fn receive_files(socket: UtpSocket, file_path: &str) {
    unimplemented!();
}

fn usage(program: &str, opts: Options) {
    let brief = "usage:\tucp [-h] [-v] [[user@]host1:]srcfile [[user@]host2:]destfile";
    print!("{}", opts.usage(&brief));
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let prog_name = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "verbose", "more debugging messages");
    opts.optflag("r", "recursive", "whether to recursively transfer files");
    opts.optopt("f", "", "file or dir to read from (server side)", "FILE");
    opts.optopt("t", "", "file or dir to write to (server side)", "FILE");
    opts.optopt("d", "", "read/write a dir instead of file (server side)", "FILE");
    opts.optopt("l", "", "", "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { println!("Error parsing args!"); usage(&prog_name, opts); exit(-1); }
    };

    let verbose: bool = matches.opt_present("v");

    if matches.opt_present("h") {
        usage(&prog_name, opts);
        return;
    }

    // First handle the server-side cases
    if matches.opt_present("f") && matches.opt_present("t") {
        println!("Can't be both server modes at the same time!");
        exit(-1);
    }

    if matches.opt_present("f") {
        unimplemented!();
        //serve(&matches.opt_str("f").unwrap(), false);
    }
    if matches.opt_present("t") {
        unimplemented!();
        //serve(&matches.opt_str("t").unwrap(), true);
    }

    // Then the user-driven (local) side
    if matches.free.len() != 2 {
        println!("Expected a single source and single destination");
        println!("");
        usage(&prog_name, opts);
        return;
    }

    let srcfile = matches.free[0].clone();
    let destfile = matches.free[1].clone();

    for fname in vec![&srcfile, &destfile] {
        if fname.match_indices(":").count() > 2 {
            println!("Invalid host/file identifier: {}", fname);
        }
    }


    match (srcfile.contains(":"), destfile.contains(":")) {
        (true, true)    => { println!("Can't have src and dest both be remote!"); return; },
        (false, false)  => { println!("One of src or dest should be remote!"); return; },
        (true , false)  => {
            let is_recv = true;
            let local_file = destfile;
            let spl: Vec<&str> = srcfile.split(":").collect();
            let host = spl[0];
            let remote_file = spl[1];
            println!("host: {}", host);
            },
        (false, true)   => {
            let is_recv = false;
            let remote_file = srcfile;
            let spl: Vec<&str> = destfile.split(":").collect();
            let host = spl[0];
            let local_file = spl[1];
            println!("host: {}", host);
            },
    }
}
