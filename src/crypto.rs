
use std::{u8, u32};
use std::io;
use std::cmp::min;
use std::io::{Read,Write, ErrorKind};
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::secretbox::{Key, Nonce};
use rustc_serialize::base64::{ToBase64, FromBase64, STANDARD};
use std::mem::transmute;

// TODO: handle case of splitting up writes > 2^32 bytes into multiple small writes
const CHUNK_SIZE: usize = 1024*64;

pub struct SecretStream<S: Read+Write> {
    pub read_nonce: Nonce,
    pub write_nonce: Nonce,
    pub key: Key,
    inner: S,
    read_buf: [u8; CHUNK_SIZE+512],
    read_buf_offset: usize,
    read_buf_len: usize,
}

impl<S: Read+Write> SecretStream<S> {
    pub fn new(stream: S) -> SecretStream<S> {
        SecretStream {
            inner: stream,
            read_nonce: secretbox::gen_nonce(),
            write_nonce: secretbox::gen_nonce(),
            key: secretbox::gen_key(),
            read_buf: [0; CHUNK_SIZE+512],
            read_buf_offset: 0,
            read_buf_len: 0,
        }
    }
}

impl<S: Read+Write> Read for SecretStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {

        // First try to return any extra older decrypted data
        if self.read_buf_len > 0 {
            let rlen = min(self.read_buf_len, buf.len());
            buf[..rlen].clone_from_slice(
                &self.read_buf[self.read_buf_offset..(self.read_buf_offset+rlen)]);
            self.read_buf_offset += rlen;
            self.read_buf_len -= rlen;
            return Ok(rlen);
        }

        let mut header_buf = [0; 4];
        try!(self.inner.read_exact(&mut header_buf));
        let len: u32 = unsafe { transmute(header_buf) };
        let len = u32::from_be(len);
        let len = len as usize;
        if len as usize > self.read_buf.len() {
            return Err(io::Error::new(ErrorKind::Other,
                format!("Message too big ({})", len)));
        }
        try!(self.inner.read_exact(&mut self.read_buf[..len]));
        /*
        println!("DECRYPT:");
        println!("\tlen: {}", len);
        println!("\tmsg: {:?}", &self.read_buf[..len]);
        println!("\tnonce: {}", nonce2string(&self.write_nonce));
        println!("\tkey: {}", key2string(&self.key));
        */
        let cleartext = match secretbox::open(&self.read_buf[..len], &self.read_nonce, &self.key) {
            Ok(cleartext) => cleartext,
            Err(_) => { return Err(io::Error::new(ErrorKind::InvalidData,
                "Failed to decrypt message (could mean corruption or malicious attack"))},
        };
        self.read_nonce.increment_le_inplace();
        let clen = cleartext.len() as usize;

        // Do we have more data than we can return this type? If so buffer it
        if clen > buf.len() {
            let buf_len = buf.len();
            buf.clone_from_slice(&cleartext[..buf_len]);
            self.read_buf[..(clen-buf_len)].clone_from_slice(&cleartext[buf_len..]);
            self.read_buf_offset = 0;
            self.read_buf_len = clen - buf_len;
            return Ok(buf_len);
        } else {
            buf[..clen].clone_from_slice(&cleartext[..clen]);
            return Ok(clen as usize);
        }
    }
}

impl<S: Read+Write> Write for SecretStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        assert!(buf.len() < u32::MAX as usize);
        let raw_len = buf.len() as u32;
        let ciphertext = secretbox::seal(buf, &self.write_nonce, &self.key);

        let len = ciphertext.len() as u32;
        let header_buf: [u8; 4] = unsafe { transmute(len.to_be()) };
        try!(self.inner.write_all(&header_buf));

        /*
        println!("DECRYPT:");
        println!("\tlen: {}", len);
        println!("\tmsg: {:?}", ciphertext);
        println!("\tnonce: {}", nonce2string(&self.write_nonce));
        println!("\tkey: {}", key2string(&self.key));
        let check = secretbox::open(&ciphertext, &self.write_nonce, &self.key).unwrap();
        */

        self.write_nonce.increment_le_inplace();
        try!(self.inner.write_all(&ciphertext[..]));
        return Ok(raw_len as usize);
    }

    fn flush(&mut self) -> io::Result<()> {
        return self.inner.flush();
    }
}

pub fn key2string(key: &Key) -> String {
    return (&(key[..])).to_base64(STANDARD);
}

pub fn string2key(s: &str) -> Result<Key, String> {
    return Ok(Key::from_slice(&s.as_bytes().from_base64().unwrap()).unwrap());
}

pub fn nonce2string(nonce: &Nonce) -> String {
    return (&(nonce[..])).to_base64(STANDARD);
}

pub fn string2nonce(s: &str) -> Result<Nonce, String> {
    return Ok(Nonce::from_slice(&s.as_bytes().from_base64().unwrap()).unwrap());
}
