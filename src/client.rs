
use super::common;

use std::io::Write;
use std::string::String;
use std::str::{self, FromStr};
use std::env;
use std::net;
use std::process;
use std::process::exit;
use std::process::Command;
use getopts::Options;
use udt::{self, UdtSocket};
use udt_extras::{UdtStream};
use crypto::{SecretStream, key2string, string2key, nonce2string, string2nonce};
use sodiumoxide::crypto::secretbox;

pub fn run_client(host: &str, local_file: &str, remote_file: &str, remote_is_dir: bool, is_recv: bool, no_crypto: bool) -> Result<(), String> {
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
    if no_crypto {
        ssh_cmd.arg("--no-crypto");
    }

    let ssh_output = ssh_cmd.output().expect("couldn't get SSH sub-process output");

    if !ssh_output.status.success() {
        return Err(format!("Error on remote end: {}",
                           String::from_utf8_lossy(&ssh_output.stderr)));
    }

    let reply = String::from_utf8_lossy(&ssh_output.stdout);
    //println!("SSH reply: {}", reply);
    let words: Vec<&str> = reply.split_whitespace().collect();
    if words.len() != 7 || words[0] != "UCP" || words[1] != "CONNECT" {
        return Err("Unexpected data via SSH pipe (TCP)".to_string());
    }
    let remote_host = words[2];
    let remote_port = words[3].parse::<u16>().expect("failed to parse remote port number");
    let remote_secret = words[4];
    let remote_read_nonce = words[5];
    let remote_write_nonce = words[6];

    println!("Got remote details:");
    println!("\tport: {}", remote_port);
    println!("\thost: {}", remote_host);
    println!("\tsecret key: {}", remote_secret);
    println!("\tsecret read nonce: {}", remote_read_nonce);
    println!("\tsecret write nonce: {}", remote_write_nonce);

    let addr = net::IpAddr::from_str(remote_host).unwrap();
    let mut socket = UdtSocket::new(udt::SocketFamily::AFInet, udt::SocketType::Stream).unwrap();
    match socket.connect(net::SocketAddr::new(addr, remote_port)) {
        Ok(_) =>  { println!("Connected."); },
        Err(e) => { return Err(e.err_msg); },
    };
    let mut stream: UdtStream = UdtStream::new(socket);

    if !no_crypto {
        let mut stream = SecretStream::new(stream);
        stream.key = string2key(remote_secret).unwrap();
        stream.read_nonce = string2nonce(remote_write_nonce).unwrap();
        stream.write_nonce = string2nonce(remote_read_nonce).unwrap();
        if is_recv {
            common::sink_files(&mut stream, local_file, remote_is_dir)
        } else {
            common::source_files(&mut stream, local_file, remote_is_dir)
        }
    } else {
        if is_recv {
            common::sink_files(&mut stream, local_file, remote_is_dir)
        } else {
            common::source_files(&mut stream, local_file, remote_is_dir)
        }
    }
}

fn usage_client(opts: Options) {
    let brief = "usage:\tucp client ..."; // XXX:
    println!("");
    println!("IMPORTANT: this is the client mode of ucp. Unless you are developing/debugging, you probably want the 'regular' one (from the 'client' from you command)");
    print!("{}", opts.usage(&brief));
}

pub fn main_client() {

    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    //opts.optflag("v", "verbose", "more debugging messages");
    opts.optflag("d", "dir-mode", "read/write a dir instead of file (client side)");
    opts.optopt("f", "from", "file or dir to read from (client side)", "FILE");
    opts.optopt("t", "to", "file or dir to write to (client side)", "FILE");
    opts.reqopt("", "host", "remote hostname to connect to", "HOSTNAME");
    opts.reqopt("", "port", "remote port to connect to", "PORT");
    opts.optopt("", "read-nonce", "secret read nonce", "NONCE");
    opts.optopt("", "write-nonce", "secret write nonce", "NONCE");
    opts.optopt("", "key", "secret key", "NONCE");
    opts.optflag("", "no-crypto", "sends data in the clear (no crypto or verification)");

    assert!(args.len() >= 2 && args[1] == "client");
    let matches = match opts.parse(&args[2..]) {
        Ok(m) => { m }
        Err(f) => { println!("{}", f.to_string()); usage_client(opts); exit(-1); }
    };

    if matches.opt_present("h") {
        usage_client(opts);
        return;
    }

    //let verbose: bool = matches.opt_present("v");
    let dir_mode: bool = matches.opt_present("d");
    let no_crypto: bool = matches.opt_present("no-crypto");

    if !no_crypto {
        if !matches.opt_present("key") ||
                !matches.opt_present("read-nonce") ||
                !matches.opt_present("write-nonce") {
            println!("If not --no-crypto, --key and --read-nonce and --write-nonce are required.");
            usage_client(opts);
            exit(-1);
        }
    }

    match (matches.opt_present("f"), matches.opt_present("t")) {
        (true, true) | (false, false) => {
            println!("Must be either 'from' or 'to', but not both");
            exit(-1);
            },
        _ => {},
    }

    let remote_host = matches.opt_str("host").unwrap();
    let remote_port = matches.opt_str("port").unwrap().parse::<u16>().unwrap();
    let addr = net::IpAddr::from_str(&remote_host).unwrap();
    let mut socket = UdtSocket::new(udt::SocketFamily::AFInet, udt::SocketType::Stream).unwrap();
    socket.connect(net::SocketAddr::new(addr, remote_port)).unwrap();;
    let mut stream: UdtStream = UdtStream::new(socket);
    println!("opened socket");

    let mut ret: Result<(), String>;
    if !no_crypto {
        let mut stream = SecretStream::new(stream);
        stream.key = string2key(&matches.opt_str("key").unwrap()).unwrap();
        stream.read_nonce = string2nonce(&matches.opt_str("read-nonce").unwrap()).unwrap();
        stream.write_nonce = string2nonce(&matches.opt_str("write-nonce").unwrap()).unwrap();
        if matches.opt_present("f") {
            ret = common::source_files(&mut stream, &matches.opt_str("f").unwrap(), dir_mode);
        } else if matches.opt_present("t") {
            ret = common::sink_files(&mut stream, &matches.opt_str("t").unwrap(), dir_mode);
        } else {
            ret = Err("Didn't Run".to_string());
        }
    } else {
        if matches.opt_present("f") {
            ret = common::source_files(&mut stream, &matches.opt_str("f").unwrap(), dir_mode);
        } else if matches.opt_present("t") {
            ret = common::sink_files(&mut stream, &matches.opt_str("t").unwrap(), dir_mode);
        } else {
            ret = Err("Didn't Run".to_string());
        }
    }

    match ret {
        Ok(_) => { exit(0); },
        Err(msg) => {
            writeln!(&mut ::std::io::stderr(), "{}", msg).unwrap();
            exit(-1);
        }
    }
}
