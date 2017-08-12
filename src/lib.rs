//! Temporary directories of files.
//!
//! The [`TempDir`] type creates a directory on the file system that
//! is deleted once it goes out of scope. At construction, the
//! `TempDir` creates a new directory with a randomly generated name
//! and a prefix of your choosing.
//!
//! [`TempDir`]: struct.TempDir.html
//! [`std::env::temp_dir()`]: https://doc.rust-lang.org/std/env/fn.temp_dir.html
//!
//! # Examples
//!
//! ```
//! extern crate tempfile;
//!
//! use std::fs::File;
//! use std::io::{self, Write};
//! use tempfile::TempDir;
//!
//! fn main() {
//!     if let Err(_) = run() {
//!         ::std::process::exit(1);
//!     }
//! }
//!
//! fn run() -> Result<(), io::Error> {
//!     // Create a directory inside of `std::env::temp_dir()`, named with
//!     // the prefix "example".
//!     let tmp_dir = TempDir::new("example")?;
//!     let file_path = tmp_dir.path().join("my-temporary-note.txt");
//!     let mut tmp_file = File::create(file_path)?;
//!     writeln!(tmp_file, "Brian was here. Briefly.")?;
//!
//!     // By closing the `TempDir` explicitly, we can check that it has
//!     // been deleted successfully. If we don't close it explicitly,
//!     // the directory will still be deleted when `tmp_dir` goes out
//!     // of scope, but we won't know whether deleting the directory
//!     // succeeded.
//!     drop(tmp_file);
//!     tmp_dir.close()?;
//!     Ok(())
//! }
//! ```
//! 
//! Securely create and manage temporary files. Temporary files created by this create are
//! automatically deleted.
//!
//! This crate provides two temporary file variants: a `tempfile()` function that returns standard
//! `File` objects and `NamedTempFile`. When choosing between the variants, prefer `tempfile()`
//! unless you either need to know the file's path or to be able to persist it.
//!
//! # Example
//!
//! ```
//! use tempfile::tempfile;
//! use std::fs::File;
//!
//! let mut file: File = tempfile().expect("failed to create temporary file");
//! ```
//!
//! # Differences
//!
//! ## Resource Leaking
//!
//! `tempfile()` will (almost) never fail to cleanup temporary files but `NamedTempFile` will if
//! its destructor doesn't run. This is because `tempfile()` relies on the OS to cleanup the
//! underlying file so the file while `NamedTempFile` relies on its destructor to do so.
//!
//! ## Security
//!
//! In the presence of pathological temporary file cleaner, relying on file paths is unsafe because
//! a temporary file cleaner could delete the temporary file which an attacker could then replace.
//!
//! `tempfile()` doesn't rely on file paths so this isn't an issue. However, `NamedTempFile` does
//! rely on file paths.
//!

extern crate rand;
extern crate remove_dir_all;

#[cfg(unix)]
extern crate libc;

#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
extern crate kernel32;

const NUM_RETRIES: u32 = 1 << 31;
const NUM_RAND_CHARS: usize = 6;

mod dir;
mod file;

pub use file::named::{NamedTempFile, NamedTempFileOptions, PersistError};
pub use file::unnamed::{TempFile};
pub use dir::{TempDir};
