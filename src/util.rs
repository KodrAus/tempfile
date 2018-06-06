use rand;
use rand::Rng;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{io, iter};

use super::PathStr;

fn tmpname(prefix: PathStr, suffix: PathStr, rand_len: usize) -> OsString {
    // If both the prefix and suffix are valid utf8
    // we can avoid allocating a temporary buffer
    match (prefix, suffix) {
        (PathStr::Utf8(prefix), PathStr::Utf8(suffix)) => {
            let mut buf = String::with_capacity(prefix.len() + suffix.len() + rand_len);
        
            buf.push_str(prefix);
            buf.extend(iter::repeat('X').take(rand_len));
            buf.push_str(suffix);
        
            // Randomize.
            fill_utf8(unsafe {
                &mut buf.as_mut_vec()[prefix.len()..prefix.len() + rand_len]
            });
            
            OsString::from(buf)
        },
        // If the prefix and suffix aren't known to be valid utf8
        // we need a temporary buffer to write the random part of
        // the name into
        (prefix, suffix) => {
            let prefix = prefix.as_ref();
            let suffix = suffix.as_ref();

            // Randomize.
            let mut rand = {
                let mut bytes = Vec::with_capacity(rand_len);
                fill_utf8(&mut bytes);
                
                unsafe { OsString::from(String::from_utf8_unchecked(bytes)) }
            };
            
            let mut buf = OsString::with_capacity(prefix.len() + suffix.len() + rand.len());
            
            buf.push(prefix);
            buf.push(rand);
            buf.push(suffix);
            
            buf
        },
    }
}

fn fill_utf8(bytes: &mut [u8]) {
    rand::thread_rng().fill_bytes(bytes);

    // We guarantee utf8.
    for byte in bytes.iter_mut() {
        *byte = match *byte % 62 {
            v @ 0...9 => (v + b'0'),
            v @ 10...35 => (v - 10 + b'a'),
            v @ 36...61 => (v - 36 + b'A'),
            _ => unreachable!(),
        }
    }
}

pub(super) fn create_helper<F, R>(
    base: &Path,
    prefix: PathStr,
    suffix: PathStr,
    random_len: usize,
    f: F,
) -> io::Result<R>
where
    F: Fn(PathBuf) -> io::Result<R>,
{
    for _ in 0..::NUM_RETRIES {
        let path = base.join(tmpname(prefix, suffix, random_len));
        return match f(path) {
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            res => res,
        };
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "too many temporary files exist",
    ))
}
