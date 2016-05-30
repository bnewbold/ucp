
// XXX: re-enable these warnings
#![allow(unused_imports, unused_variables, unused_mut)]

extern crate getopts;
extern crate utp;

mod client;
mod server;
mod common;

use std::str;
use std::env;
use std::process::exit;
use getopts::Options;
use utp::{UtpSocket, UtpListener};

fn usage(opts: Options) {
    let brief = "usage:\tucp [-h] [-v] [[user@]host1:]srcfile [[user@]host2:]destfile";
    print!("{}", opts.usage(&brief));
}

fn main() {

    let args: Vec<String> = env::args().collect();

    // First check for "hidden" server mode
    if args.len() > 1 && args[1] == "server" {
        server::main_server();
        return;
    }

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    //opts.optflag("v", "verbose", "more debugging messages");
    opts.optflag("r", "recursive", "whether to recursively transfer files (directory)");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { println!("Error parsing args: {}", f); usage(opts); exit(-1); }
    };

    //let verbose: bool = matches.opt_present("v");
    let recursive: bool = matches.opt_present("r");

    if matches.opt_present("h") {
        usage(opts);
        return;
    }

    if matches.free.len() != 2 {
        println!("Expected a single source and single destination");
        println!("");
        usage(opts);
        return;
    }

    let srcfile = matches.free[0].clone();
    let destfile = matches.free[1].clone();

    for fname in vec![&srcfile, &destfile] {
        if fname.match_indices(":").count() > 2 {
            println!("Invalid host/file identifier: {}", fname);
            exit(-1);
        }
    }


    match (srcfile.contains(":"), destfile.contains(":")) {
        (true, true)    => { println!("Can't have src and dest both be remote!"); exit(-1); },
        (false, false)  => { println!("One of src or dest should be remote!"); exit(-1); },
        (true , false)  => {
            let is_recv = true;
            let local_file = &destfile;
            let spl: Vec<&str> = srcfile.split(":").collect();
            let host = spl[0];
            let remote_file = spl[1];
            client::run_client(host, local_file, remote_file, recursive, is_recv);
            },
        (false, true)   => {
            let is_recv = false;
            let remote_file = &srcfile;
            let spl: Vec<&str> = destfile.split(":").collect();
            let host = spl[0];
            let local_file = spl[1];
            client::run_client(host, local_file, remote_file, recursive, is_recv);
            },
    }
}
