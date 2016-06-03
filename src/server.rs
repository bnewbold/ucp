
extern crate daemonize;
extern crate syslog;

use super::common;

use std::io::Write;
use std::str::{self, FromStr};
use std::env;
use std::net;
use std::io;
use std::error::Error;
use std::env::home_dir;
use std::process::exit;
use log;
use env_logger;
use getopts::Options;
use udt::{self, UdtSocket, UdtStatus};
use crypto::{SecretStream, key2string, string2key, nonce2string, string2nonce};
use udt_extras::{UdtStream};
use sodiumoxide::crypto::secretbox;

pub fn get_local_ip() -> Result<net::IpAddr, String> {
    let ip_str = match env::var("SSH_CONNECTION") {
        Ok(val) => {
            match val.split(' ').nth(2) {
                Some(x) => x.to_string(),
                None => { return Err(format!("Failed to parse $SSH_CONNECTION: {}", val.to_string())); },
            }
        },
        Err(_) => {
            warn!("Can't find $SSH_CONNECTION; running locally? Falling back to 127.0.0.1");
            "127.0.0.1".to_string()
        },
    };

    // First try IPv4
    match net::Ipv4Addr::from_str(&ip_str) {
        Ok(x) => { return Ok(net::IpAddr::V4(x)) },
        Err(_) => (),
    };
    // Then IPv6
    match net::Ipv6Addr::from_str(&ip_str) {
        Ok(x) => { return Ok(net::IpAddr::V6(x)) },
        Err(e) => { return Err(e.description().to_string()); },
    };
}


fn run_server(path: &str, is_recv: bool, recursive: bool, daemonize: bool, no_crypto: bool) -> Result<(), String> {

    // Connect to an hypothetical local server running on port 61000
    let listen_addr = get_local_ip().unwrap();

    // Actually accept connection from anybody
    let all_addr = net::IpAddr::V4(net::Ipv4Addr::from_str("0.0.0.0").unwrap());

    let listener = UdtSocket::new(udt::SocketFamily::AFInet, udt::SocketType::Stream).unwrap();

    for port in 61000..62000 {
        match listener.bind(net::SocketAddr::new(all_addr, port)) {
            Ok(_) => { break; },
            Err(e) => {
                if e.err_code != 1003 {
                    // Code 1003 is "can't get port", meaning it's taken
                    return Err(format!("Error binding {}: {}:  {}", port, e.err_code, e.err_msg));
                };
            }
        }
    }

    if listener.getstate() != UdtStatus::OPENED {
        println!("{:?}", listener.getstate());
        return Err("Couldn't bind to *any* valid port".to_string());
    }

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

    info!("listening on {}:{}", listen_addr, listen_port);

    // Send back (via SSH stdout) details so client can connect
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
        info!("Not daemonizing (DEBUG MODE)");
    }
    let (mut socket, _src) = listener.accept().unwrap();
    info!("Got connection from {}", socket.getpeername().unwrap());
    let mut stream: UdtStream = UdtStream::new(socket);

    if !no_crypto {
        let mut stream = SecretStream::new(stream);
        stream.key = secret_key;
        stream.read_nonce = read_nonce;
        stream.write_nonce = write_nonce;
        if is_recv {
            common::sink_files(&mut stream, path, recursive)
        } else {
            common::source_files(&mut stream, path, recursive)
        }
    } else {
        if is_recv {
            common::sink_files(&mut stream, path, recursive)
        } else {
            common::source_files(&mut stream, path, recursive)
        }
    }
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
    opts.optflag("v", "verbose", "more debugging messages");
    opts.optflag("d", "dir-mode", "read/write a dir instead of file (server side)");
    opts.optflag("", "no-daemonize", "don't daemonize (for debuggign)");
    opts.optopt("f", "from", "file or dir to read from (server side)", "FILE");
    opts.optopt("t", "to", "file or dir to write to (server side)", "FILE");
    opts.optflag("", "no-crypto", "sends data in the clear (no crypto or verification)");

    assert!(args.len() >= 2 && args[1] == "server");
    let matches = match opts.parse(&args[2..]) {
        Ok(m) => { m }
        Err(f) => { println!("{}", f.to_string()); usage_server(opts); exit(-1); }
    };

    if matches.opt_present("h") {
        usage_server(opts);
        return;
    }

    let verbose: bool = matches.opt_present("v");
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

    // Set up logging; syslog doesn't have a thingy
    if daemonize {
        match syslog::unix(syslog::Facility::LOG_USER) {
            Ok(logger) => {
                log::set_logger(|max_log_level| {
                    max_log_level.set(log::LogLevelFilter::Info);
                    logger
                }).unwrap();
            },
            Err(_) => { 
                println!("Couldn't connect to syslog (and in daemonize mode");
                exit(-1);
            }
        }
    } else {
        let mut builder = env_logger::LogBuilder::new();
        builder.parse("INFO");
        if env::var("RUST_LOG").is_ok() {
            builder.parse(&env::var("RUST_LOG").unwrap());
        }
        builder.init().unwrap();
    }

    let (file_name, is_recv) = if matches.opt_present("f") {
        (matches.opt_str("f").unwrap(), false)
    } else {
        (matches.opt_str("t").unwrap(), true)
    };
    match run_server(&file_name, is_recv, dir_mode, daemonize, no_crypto) {
        Ok(_) => { exit(0); },
        Err(msg) => {
            error!("{}", msg);
            exit(-1);
        }
    }
}
