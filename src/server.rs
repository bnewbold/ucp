
extern crate getopts;
extern crate utp;
extern crate daemonize;

use super::common;

use std::str;
use std::env;
use std::env::home_dir;
use std::process::exit;
use getopts::Options;
use utp::{UtpSocket, UtpListener};

fn run_server(path: &str, is_recv: bool, recursive: bool) {

    // TODO: try to detect the address the SSH connection came in on via the SSH_CONNECTION env
    // variable.

    // Connect to an hypothetical local server running on port 61000
    let addr = "127.0.0.1:61000";

    // Accept connection from anybody
    let listener = UtpListener::bind(addr).expect("Error binding to local port");

    let listen_port = listener.local_addr().unwrap().port();
    let listen_addr = listener.local_addr().unwrap().ip();

    // Send back details so client can connect
    println!("UDP CONNECT {} {} {}", listen_addr, listen_port, "<SECRET>");

    // TODO: maybe wait for an ACK of some sort here before daemonizing?

    // At this point we partially daemonize (fork and redirect terminal), so that SSH will drop us.
    // But, don't clobber working dir.
    let working_dir = match env::home_dir() {
        Some(path) => path,
        None => env::current_dir().unwrap(),
    };
    // XXX: should maybe use log/syslog from here on?
    let daemonizer = daemonize::Daemonize::new().working_directory(working_dir);

    match daemonizer.start() {
         Ok(_) => println!("Success, daemonized"),
         Err(e) => println!("{}", e),
     }

    let (mut socket, _src) = listener.accept().unwrap();
    println!("Got connection from {}", socket.peer_addr().unwrap());
    let mut stream = socket.into();

    if is_recv {
        common::sink_files(&mut stream, path, recursive);
    } else {
        common::source_files(&mut stream, path, recursive);
    }
    stream.close().unwrap();
}

fn usage_server(opts: Options) {
    let brief = "usage:\tucp server ..."; // XXX:
    println!("");
    println!("IMPORTANT: this is the server mode of ucp. Unless you are developing/debugging, you probably want the 'regular' one (from the 'server' from you command)");
    print!("{}", opts.usage(&brief));
}

pub fn main_server() {

    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    //opts.optflag("v", "verbose", "more debugging messages");
    opts.optflag("d", "dir-mode", "read/write a dir instead of file (server side)");
    opts.optopt("f", "from", "file or dir to read from (server side)", "FILE");
    opts.optopt("t", "to", "file or dir to write to (server side)", "FILE");

    assert!(args.len() >= 2 && args[1] == "server");
    let matches = match opts.parse(&args[2..]) {
        Ok(m) => { m }
        Err(f) => { println!("Error parsing args!"); usage_server(opts); exit(-1); }
    };

    if matches.opt_present("h") {
        usage_server(opts);
        return;
    }

    //let verbose: bool = matches.opt_present("v");
    let dir_mode: bool = matches.opt_present("d");

    match (matches.opt_present("f"), matches.opt_present("t")) {
        (true, true) | (false, false) => {
            println!("Must be either 'from' or 'to', but not both");
            exit(-1);
            },
        _ => {},
    }

    if matches.opt_present("f") {
        run_server(&matches.opt_str("f").unwrap(), false, dir_mode);
    }
    if matches.opt_present("t") {
        run_server(&matches.opt_str("t").unwrap(), true, dir_mode);
    }
}
