
use std::str;
use std::env;
use std::path::Path;
use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::io;
use std::io::{Read, Write, BufRead, BufReader};
use std::process::exit;
use std::net;

fn fake_io_error(msg: &str) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Other, msg))
}

const CHUNK_SIZE: usize = 1024*16;

pub fn source_files<S: Read + Write>(stream: &mut S, file_path: &str, recursive: bool) -> io::Result<()> {
    println!("SOURCE FILE: {}", file_path);
    if recursive { unimplemented!(); }
    let mut f = try!(File::open(file_path));
    let metadata = try!(f.metadata());
    assert!(metadata.is_file());
    let fmode: u32 = metadata.permissions().mode();
    let flen: usize = metadata.len() as usize;

    // Format as 4 digits octal, left-padding with zero
    let line = format!("C{:0>4o} {} {}\n", fmode, flen, file_path);
    try!(stream.write_all(line.as_bytes()));
    println!("{}", line);

    let mut byte_buf = [0; 1];
    try!(stream.read_exact(&mut byte_buf));
    let reply = byte_buf[0];
    match reply {
        0 => {},    // Success, pass
        1 | 2 => {      // Warning
            unimplemented!();
        },
        _ => { return fake_io_error("Unexpected status char!"); },
    };

    let mut buf = [0; CHUNK_SIZE];
    let mut sent: usize = 0;
    while sent < flen {
        let rlen = try!(f.read(&mut buf));
        assert!(rlen > 0);
        let mut wbuf = &mut buf[..rlen];
        try!(stream.write_all(&wbuf));
        sent += rlen;
        //println!("sent: {}", sent);
    }
    // f.close(); XXX:
    try!(stream.read_exact(&mut byte_buf));
    let reply = byte_buf[0];
    match reply {
        0 => {},    // Success, pass
        1 | 2 => {      // Warning
            unimplemented!();
        },
        _ => { return fake_io_error("Unexpected status char!"); },
    };
    Ok(())
}

fn raw_read_line<S: Read + Write>(stream: &mut S) -> io::Result<String> {

    let mut s = String::new();
    let mut byte_buf = [0];
    loop {
        try!(stream.read_exact(&mut byte_buf));
        if byte_buf[0] == '\n' as u8 {
            return Ok(s);
        }
        s.push(byte_buf[0] as char);
    }
}

// TODO: it would be nice to be able to do BufReader/BufWriter on stream. This would require
// implementations of Read and Write on immutable references to stream (a la TcpStream, File, et
// al)
pub fn sink_files<S: Read + Write>(stream: &mut S, file_path: &str, recursive: bool) -> io::Result<()> {
    info!("SINK FILE: {}", file_path);
    if recursive { unimplemented!(); }
    let mut f = try!(File::create(file_path));

    let mut byte_buf = [0; 1];
    let mut buf = [0; CHUNK_SIZE];
    try!(stream.read_exact(&mut byte_buf));
    let msg_type = byte_buf[0];
    match msg_type as char {
        'C' => {
            info!("Going to create!");
        },
        'D' => { unimplemented!(); },
        'E' => { unimplemented!(); },
        'T' => { unimplemented!(); },
        _   => { return fake_io_error(&format!("Unexpected message type: {}", msg_type)); },
    };
    let line = try!(raw_read_line(stream));
    println!("got msg: {}", line);
    try!(stream.write(&[0]));
    let line: Vec<&str> = line.split_whitespace().collect();
    assert!(line.len() == 3);
    let fmode: u32 = match u32::from_str_radix(line[0], 8) {
        Ok(x) => x,
        Err(_) => { return fake_io_error("unparsable file mode in ucp 'C' message"); },
    };
    let flen: usize = match line[1].parse::<usize>() {
        Ok(x) => x,
        Err(_) => { return fake_io_error("unparsable file length in ucp 'C' message"); },
    };
    let fpath = Path::new(line[2]);

    // TODO: I've disabled set_len; is this best practice? scp doesn't do it.
    //try!(f.set_len(flen as u64));
    try!(fs::set_permissions(file_path, PermissionsExt::from_mode(fmode)));

    let mut received: usize = 0;
    while received < flen {
        let rlen = try!(stream.read(&mut buf));
        //println!("recieved: {}", rlen);
        assert!(rlen > 0);
        let mut wbuf = &mut buf[..rlen];
        try!(f.write_all(&wbuf));
        received += rlen;
    }
    try!(f.sync_all());
    // f.close(); XXX: closes automatically?
    try!(stream.write(&[0]));
    Ok(())
}
