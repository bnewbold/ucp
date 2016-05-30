
use std::{u8, u32};
use std::io;
use std::io::{Read,Write, ErrorKind};
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::secretbox::{Key, Nonce};
use rustc_serialize::base64::{ToBase64, FromBase64, STANDARD};
use std::mem::transmute;

// TODO: handle case of splitting up writes > 2^32 bytes into multiple small writes

pub struct SecretStream<S: Read+Write> {
    read_nonce: Nonce,
    write_nonce: Nonce,
    pub key: Key,
    inner: S,
}

impl<S: Read+Write> SecretStream<S> {
    pub fn new(stream: S) -> SecretStream<S> {
        SecretStream {
            inner: stream,
            read_nonce: secretbox::gen_nonce(),
            write_nonce: secretbox::gen_nonce(),
            key: secretbox::gen_key(),
        }
    }
}

impl<S: Read+Write> Read for SecretStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut header_buf = [0; 4];
        try!(self.inner.read_exact(&mut header_buf));
        let len: u32 = unsafe { transmute(header_buf) };
        let len = len.to_be();
        if len as usize > buf.len() {
            return Err(io::Error::new(ErrorKind::Other,
                format!("Buffer not big enough ({} < {})", buf.len(), len)));
        }
        try!(self.inner.read_exact(buf));
        let cleartext = match secretbox::open(buf, &self.read_nonce, &self.key) {
            Ok(cleartext) => cleartext,
            Err(_) => { return Err(io::Error::new(ErrorKind::InvalidData,
                "Failed to decrypt message (could mean corruption or malicious attack"))},
        };
        self.read_nonce.increment_le_inplace();
        let len = len as usize;
        buf.clone_from_slice(&cleartext[..len]);
        return Ok(len as usize);
    }
}

impl<S: Read+Write> Write for SecretStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        assert!(buf.len() < u32::MAX as usize);
        let len = buf.len() as u32;
        let header_buf: [u8; 4] = unsafe { transmute(len.to_be()) };
        try!(self.inner.write_all(&header_buf));
        let ciphertext = secretbox::seal(buf, &self.write_nonce, &self.key);
        self.write_nonce.increment_le_inplace();
        try!(self.inner.write_all(&ciphertext[..]));
        return Ok(len as usize);
    }

    fn flush(&mut self) -> io::Result<()> {
        return self.inner.flush();
    }
}

pub fn key2string(key: &Key) -> String {
    return (&(key[..])).to_base64(STANDARD);
}

pub fn string2key(s: &str) -> Result<Key, String> {
    println!("KEYBYTES: {}", secretbox::KEYBYTES);
    return Ok(Key::from_slice(&s.as_bytes().from_base64().unwrap()).unwrap());
}

