// Original Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at http://rust-lang.org/COPYRIGHT.
// Modications by Paul Klinger

use std::io::BufRead;

extern crate memchr;

use std::io::{ErrorKind, Result};


// Same as std::io::BufRead::read_until except it breaks on multiple possible deliminators
pub fn read_until_multiple<R: BufRead + ?Sized>(
    r: &mut R,
    delims: &[u8],
    buf: &mut Vec<u8>,
) -> Result<usize> {
    let mut read = 0;
    'read_loop: loop {
        // 'fill_loop is not really a loop, just for breaking
        let (done, used) = 'fill_loop: loop {
            let available = match r.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue 'read_loop,
                Err(e) => return Err(e),
            };
            for &delim in delims {
                match memchr::memchr(delim, available) {
                    Some(i) => {
                        buf.extend_from_slice(&available[..i + 1]);
                        break 'fill_loop (true, i + 1);
                    }
                    None => {}
                }
            }
            buf.extend_from_slice(available);
            break (false, available.len());
        };

        r.consume(used);
        read += used;
        if done || used == 0 {
            return Ok(read);
        }
    }
}
