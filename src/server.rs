
extern crate daemonize;

use super::common;

use std::str::{self, FromStr};
use std::env;
use std::net;
use std::env::home_dir;
use std::process::exit;
use getopts::Options;
use udt::{self, UdtSocket};
use crypto::{SecretStream, key2string, string2key, nonce2string, string2nonce};
use udt_extras::{UdtStream};
use sodiumoxide::crypto::secretbox;

fn run_server(path: &str, is_recv: bool, recursive: bool, daemonize: bool, no_crypto: bool) {

    // TODO: try to detect the address the SSH connection came in on via the SSH_CONNECTION env
    // variable.

    // Connect to an hypothetical local server running on port 61000
    let listen_addr = common::get_local_ip().unwrap();
    let port = 61000;

    // Accept connection from anybody
    let listener = UdtSocket::new(udt::SocketFamily::AFInet, udt::SocketType::Stream).unwrap();
    listener.bind(net::SocketAddr::new(listen_addr, port)).expect("Error binding to local port");
    listener.listen(1).unwrap();

    let listen_port = listener.getsockname().unwrap().port();

    let secret_key = secretbox::gen_key();
    let read_nonce = secretbox::gen_nonce();
    let write_nonce = secretbox::gen_nonce();

    /* XXX: DEBUG:
    assert!(secret_key == string2key(&key2string(&secret_key)).unwrap());
    assert!(read_nonce == string2nonce(&nonce2string(&read_nonce)).unwrap());
    let read_nonce = secretbox::Nonce::from_slice(&[0; secretbox::NONCEBYTES]).unwrap();
    let write_nonce = secretbox::Nonce::from_slice(&[0; secretbox::NONCEBYTES]).unwrap();
    */

    // Send back details so client can connect
    println!("UCP CONNECT {} {} {} {} {}",
        listen_addr,
        listen_port,
        key2string(&secret_key),
        nonce2string(&read_nonce),
        nonce2string(&write_nonce));


    // TODO: maybe wait for an ACK of some sort here before daemonizing?

    if daemonize {
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
    } else {
        println!("Not daemonizing (DEBUG MODE)");
    }
    let (mut socket, _src) = listener.accept().unwrap();
    println!("Got connection from {}", socket.getpeername().unwrap());
    let mut stream: UdtStream = UdtStream::new(socket);

    if !no_crypto {
        let mut stream = SecretStream::new(stream);
        stream.key = secret_key;
        stream.read_nonce = read_nonce;
        stream.write_nonce = write_nonce;
        if is_recv {
            common::sink_files(&mut stream, path, recursive);
        } else {
            common::source_files(&mut stream, path, recursive);
        }
    } else {
        if is_recv {
            common::sink_files(&mut stream, path, recursive);
        } else {
            common::source_files(&mut stream, path, recursive);
        }
    }
    // XXX: does Drop do this well enough?
    //stream.close().unwrap();
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
    opts.optflag("", "no-daemonize", "don't daemonize (for debuggign)");
    opts.optopt("f", "from", "file or dir to read from (server side)", "FILE");
    opts.optopt("t", "to", "file or dir to write to (server side)", "FILE");
    opts.optflag("", "no-crypto", "sends data in the clear (no crypto or verification)");

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
    let daemonize: bool = !matches.opt_present("no-daemonize");
    let no_crypto: bool = matches.opt_present("no-crypto");

    match (matches.opt_present("f"), matches.opt_present("t")) {
        (true, true) | (false, false) => {
            println!("Must be either 'from' or 'to', but not both");
            exit(-1);
            },
        _ => {},
    }

    if matches.opt_present("f") {
        run_server(&matches.opt_str("f").unwrap(), false, dir_mode, daemonize, no_crypto);
    }
    if matches.opt_present("t") {
        run_server(&matches.opt_str("t").unwrap(), true, dir_mode, daemonize, no_crypto);
    }
}
