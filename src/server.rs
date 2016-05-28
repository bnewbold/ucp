
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

fn usage(prog_name: &str, opts: Options) {
    let brief = format!("usage:\t{} server ...", prog_name); // XXX:
    print!("{}", opts.usage(&brief));
}

pub fn main_server() {

    let args: Vec<String> = env::args().collect();
    let prog_name = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "verbose", "more debugging messages");
    opts.optflag("d", "dir-mode", "read/write a dir instead of file (server side)");
    opts.optopt("f", "from", "file or dir to read from (server side)", "FILE");
    opts.optopt("t", "to", "file or dir to write to (server side)", "FILE");

    assert!(args.len() >= 2 && args[1] == "server");
    let matches = match opts.parse(&args[2..]) {
        Ok(m) => { m }
        Err(f) => { println!("Error parsing args!"); usage(&prog_name, opts); exit(-1); }
    };

    if matches.opt_present("h") {
        usage(&prog_name, opts);
        return;
    }

    let verbose: bool = matches.opt_present("v");
    let dir_mode: bool = matches.opt_present("d");

    match (matches.opt_present("f"), matches.opt_present("t")) {
        (true, true) | (false, false) => {
            println!("Must be either 'from' or 'to', but not both");
            exit(-1);
            },
        _ => {},
    }

    if matches.opt_present("f") {
        run_server(&matches.opt_str("f").unwrap(), false);
    }
    if matches.opt_present("t") {
        unimplemented!();
        run_server(&matches.opt_str("t").unwrap(), true);
    }
}
