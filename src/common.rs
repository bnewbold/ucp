
extern crate utp;

use std::str;
use std::env;
use std::path::Path;
use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::io;
use std::io::{Read, Write, BufRead, BufReader};
use std::process::exit;
use utp::{UtpStream};


pub fn source_files(stream: &mut UtpStream, file_path: &str, recursive: bool) {
    if recursive { unimplemented!(); }
    let mut f = File::open(file_path).unwrap();
    let metadata = f.metadata().unwrap();
    assert!(metadata.is_file());
    let fmode: u32 = metadata.permissions().mode();
    let flen: usize = metadata.len() as usize;

    // Format as 4 digits octal, left-padding with zero
    let line = format!("C{:0<4o} {} {}\n", fmode, flen, file_path);
    stream.write_all(line.as_bytes()).unwrap();

    let mut byte_buf = [0; 1];
    stream.read_exact(&mut byte_buf).unwrap();
    let reply = byte_buf[0];
    match reply {
        0 => {},    // Success, pass
        1 | 2 => {      // Warning
            unimplemented!();
        },
        _ => { panic!("Unexpected status char!") },
    };

    let mut buf = [0; 4096];
    let mut sent: usize = 0;
    while sent < flen {
        let rlen = f.read(&mut buf).unwrap();
        assert!(rlen > 0);
        let mut wbuf = &mut buf[..rlen];
        stream.write_all(&wbuf).unwrap();
        sent += rlen;
    }
    // f.close(); XXX:
    stream.read_exact(&mut byte_buf).unwrap();
    let reply = byte_buf[0];
    match reply {
        0 => {},    // Success, pass
        1 | 2 => {      // Warning
            unimplemented!();
        },
        _ => { panic!("Unexpected status char!") },
    };
}

fn raw_read_line(stream: &mut UtpStream) -> io::Result<String> {

    let mut s = String::new();
    let mut byte_buf = [0];
    loop {
        stream.read_exact(&mut byte_buf).unwrap();
        if byte_buf[0] == '\n' as u8 {
            return Ok(s);
        }
        s.push(byte_buf[0] as char);
    }
}

// TODO: it would be nice to be able to do BufReader/BufWriter on UtpStream. This would require
// implementations of Read and Write on immutable references to UtpStream (a la TcpStream, File, et
// al)
pub fn sink_files(stream: &mut UtpStream, file_path: &str, recursive: bool) {
    if recursive { unimplemented!(); }
    let mut f = File::create(file_path).unwrap();

    let mut byte_buf = [0; 1];
    let mut buf = [0; 4096];
    stream.read_exact(&mut byte_buf).unwrap();
    let msg_type = byte_buf[0];
    match msg_type as char {
        'C' => {} // pass
        'D' => { unimplemented!(); },
        'E' => { unimplemented!(); },
        'T' => { unimplemented!(); },
        _   => { panic!(format!("Unexpected message type: {}", msg_type)); },
    };
    let line = raw_read_line(stream).unwrap();
    let line: Vec<&str> = line.split_whitespace().collect();
    assert!(line.len() == 3);
    let fmode: u32 = u32::from_str_radix(line[0], 8).unwrap();
    let flen: usize = line[1].parse::<usize>().unwrap();
    let fpath = Path::new(line[2]);

    f.set_len(flen as u64).unwrap();
    fs::set_permissions(file_path, PermissionsExt::from_mode(fmode)).unwrap();

    let mut received: usize = 0;
    while received < flen {
        let rlen = stream.read(&mut buf).unwrap();
        assert!(rlen > 0);
        let mut wbuf = &mut buf[..rlen];
        f.write_all(&wbuf).unwrap();
        received += rlen;
    }
    f.sync_all().unwrap();
    // f.close(); XXX: closes automatically?
    stream.write(&[0]).unwrap();
}
