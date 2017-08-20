use std::io::{self, Read, Write, Seek, SeekFrom};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::ops::{Deref, DerefMut};
use std::error;
use std::fmt;
use std::env;
use std;

mod imp;
mod util;

/// Create a new temporary file.
/// 
/// # Security
/// 
/// > TODO
/// 
/// # Resource Leaking
/// 
/// > TODO
/// 
/// # Errors
/// 
/// > TODO
/// 
/// # Examples
/// 
/// > TODO
pub fn tempfile() -> io::Result<File> {
    tempfile_in(&env::temp_dir())
}

/// Create a new temporary file in the specified directory.
/// 
/// # Security
/// 
/// > TODO
/// 
/// # Resource Leaking
/// 
/// See [`tempfile`].
/// 
/// # Errors
/// 
/// > TODO
/// 
/// # Examples
/// 
/// > TODO
pub fn tempfile_in<P: AsRef<Path>>(dir: P) -> io::Result<File> {
    imp::create(dir.as_ref())
}

/// A named temporary file.
/// 
/// The default constructor, [`NamedTempFile::new`], creates files in
/// the location returned by [`std::env::temp_dir()`], but `NamedTempFile`
/// can be configured to manage a temporary file in any location
/// by constructing with [`NamedTempFile::new_in`].
/// 
/// # Security
/// 
/// This variant is *NOT* secure/reliable in the presence of a pathological temporary file cleaner.
///
/// # Resource Leaking
/// 
/// If the program exits before the `NamedTempFile` destructor is
/// run, such as via [`std::process::exit`], by segfaulting, or by
/// receiving a signal like `SIGINT`, then the temporary file
/// will not be deleted.
///
/// Use the [`tempfile`] function unless you absolutely need a named file.
/// 
/// [`tempfile`]: fn.tempfile.html
/// [`NamedTempDir::new`]: #method.new
/// [`NamedTempDir::new_in`]: #method.new_in
/// [`std::env::temp_dir()`]: https://doc.rust-lang.org/std/env/fn.temp_dir.html
/// [`std::process::exit`]: http://doc.rust-lang.org/std/process/fn.exit.html
pub struct NamedTempFile(Option<NamedTempFileInner>);

struct NamedTempFileInner {
    file: File,
    path: PathBuf,
}

impl fmt::Debug for NamedTempFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NamedTempFile({:?})", self.inner().path)
    }
}

impl Deref for NamedTempFile {
    type Target = File;

    #[inline]
    fn deref(&self) -> &File {
        self.file()
    }
}

impl DerefMut for NamedTempFile {
    #[inline]
    fn deref_mut(&mut self) -> &mut File {
        self.file_mut()
    }
}

impl AsRef<Path> for NamedTempFile {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.path()
    }
}

/// Error returned when persisting a temporary file fails.
#[derive(Debug)]
pub struct PersistError {
    /// The underlying IO error.
    pub error: io::Error,
    /// The temporary file that couldn't be persisted.
    pub file: NamedTempFile,
}

impl From<PersistError> for io::Error {
    #[inline]
    fn from(error: PersistError) -> io::Error {
        error.error
    }
}

impl From<PersistError> for NamedTempFile {
    #[inline]
    fn from(error: PersistError) -> NamedTempFile {
        error.file
    }
}

impl fmt::Display for PersistError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to persist temporary file: {}", self.error)
    }
}

impl error::Error for PersistError {
    fn description(&self) -> &str {
        "failed to persist temporary file"
    }
    fn cause(&self) -> Option<&error::Error> {
        Some(&self.error)
    }
}

impl NamedTempFile {
    #[inline]
    fn inner(&self) -> &NamedTempFileInner {
        self.0.as_ref().unwrap()
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut NamedTempFileInner {
        self.0.as_mut().unwrap()
    }

    #[inline]
    fn take_inner(&mut self) -> NamedTempFileInner {
        self.0.take().unwrap()
    }

    /// Create a new named temporary file.
    ///
    /// See [`NamedTempFileBuilder`] for more configuration.
    /// 
    /// # Security
    /// 
    /// This will create a temporary file in the default temporary file
    /// directory (platform dependent). These directories are often patrolled by temporary file
    /// cleaners so only use this method if you're *positive* that the temporary file cleaner won't
    /// delete your file.
    ///
    /// Reasons to use this method:
    ///
    ///   1. The file has a short lifetime and your temporary file cleaner is
    ///      sane (doesn't delete recently accessed files).
    ///
    ///   2. You trust every user on your system (i.e. you are the only user).
    ///
    ///   3. You have disabled your system's temporary file cleaner or verified
    ///      that your system doesn't have a temporary file cleaner.
    ///
    /// Reasons not to use this method:
    ///
    ///   1. You'll fix it later. No you won't.
    ///
    ///   2. You don't care about the security of the temporary file. If none of
    ///      the "reasons to use this method" apply, referring to a temporary
    ///      file by name may allow an attacker to create/overwrite your
    ///      non-temporary files. There are exceptions but if you don't already
    ///      know them, don't use this method.
    /// 
    /// # Errors
    /// 
    /// > TODO
    /// 
    /// # Examples
    /// 
    /// Create a named temporary file, then persist it to a new location:
    /// 
    /// ```no_run
    /// # extern crate tempfile;
    /// use tempfile::NamedTempFile;
    /// 
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let file = NamedTempFile::new()?;
    /// 
    /// let persisted_file = file.persist("./saved_file.txt")?;
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// [`NamedTempFileBuilder`]: struct.NamedTempFileBuilder.html
    pub fn new() -> io::Result<NamedTempFile> {
        NamedTempFileBuilder::new().create()
    }

    /// Create a new temporary file in the specified directory.
    /// 
    /// # Errors
    /// 
    /// > TODO
    /// 
    /// # Examples
    /// 
    /// Create a named temporary file in the current location:
    /// 
    /// ```no_run
    /// # extern crate tempfile;
    /// use tempfile::NamedTempFile;
    /// 
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let file = NamedTempFile::new_in("./")?;
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// [`NamedTempFile::new`]: #method.new
    pub fn new_in<P: AsRef<Path>>(dir: P) -> io::Result<NamedTempFile> {
        NamedTempFileBuilder::new().create_in(dir)
    }


    /// Get the temporary file's path.
    ///
    /// # Security
    /// 
    /// Only use this method if you're positive that a
    /// temporary file cleaner won't have deleted your file. Otherwise, the path
    /// returned by this method may refer to an attacker controlled file.
    /// 
    /// # Examples
    /// 
    /// > TODO
    #[inline]
    pub fn path(&self) -> &Path {
        &self.inner().path
    }

    /// Close and remove the temporary file.
    ///
    /// Use this if you want to detect errors in deleting the file.
    /// 
    /// # Errors
    /// 
    /// > TODO
    /// 
    /// # Examples
    /// 
    /// > TODO
    pub fn close(mut self) -> io::Result<()> {
        let NamedTempFileInner { path, file } = self.take_inner();
        drop(file);
        fs::remove_file(path)
    }

    /// Persist the temporary file at the target path.
    ///
    /// If a file exists at the target path, persist will atomically replace it.
    /// If this method fails, it will return `self` in the resulting
    /// [`PersistError`].
    ///
    /// Note: Temporary files cannot be persisted across filesystems.
    ///
    /// # Security
    /// 
    /// Only use this method if you're positive that a
    /// temporary file cleaner won't have deleted your file. Otherwise, you
    /// might end up persisting an attacker controlled file.
    /// 
    /// # Errors
    /// 
    /// > TODO
    /// 
    /// # Examples
    /// 
    /// > TODO
    /// 
    /// [`PersistError`]: struct.PersistError.html
    pub fn persist<P: AsRef<Path>>(mut self, new_path: P) -> Result<File, PersistError> {
        match imp::persist(&self.inner().path, new_path.as_ref(), true) {
            Ok(_) => Ok(self.take_inner().file),
            Err(e) => {
                Err(PersistError {
                    file: self,
                    error: e,
                })
            }
        }
    }

    /// Persist the temporary file at the target path iff no file exists there.
    ///
    /// If a file exists at the target path, fail. If this method fails, it will
    /// return `self` in the resulting PersistError.
    ///
    /// Note: Temporary files cannot be persisted across filesystems. Also Note:
    /// This method is not atomic. It can leave the original link to the
    /// temporary file behind.
    ///
    /// # Security
    /// 
    /// Only use this method if you're positive that a
    /// temporary file cleaner won't have deleted your file. Otherwise, you
    /// might end up persisting an attacker controlled file.
    /// 
    /// # Errors
    /// 
    /// > TODO
    /// 
    /// # Examples
    /// 
    /// > TODO
    pub fn persist_noclobber<P: AsRef<Path>>(mut file: NamedTempFile, new_path: P) -> Result<File, PersistError> {
        match imp::persist(&file.inner().path, new_path.as_ref(), false) {
            Ok(_) => Ok(file.take_inner().file),
            Err(e) => {
                Err(PersistError {
                    file: file,
                    error: e,
                })
            }
        }
    }

    /// Reopen the temporary file.
    ///
    /// This function is useful when you need multiple independent handles to
    /// the same file. It's perfectly fine to drop the original `NamedTempFile`
    /// while holding on to `File`s returned by this function; the `File`s will
    /// remain usable. However, they may not be nameable.
    /// 
    /// # Errors
    /// 
    /// > TODO
    /// 
    /// # Examples
    /// 
    /// > TODO
    pub fn reopen(&self) -> io::Result<File> {
        imp::reopen(self.file(), NamedTempFile::path(self))
    }

    /// Get a reference to the underlying file.
    pub fn file(&self) -> &File {
        &self.inner().file
    }

    /// Get a mutable reference to the underlying file.
    pub fn file_mut(&mut self) -> &mut File {
        &mut self.inner_mut().file
    }

    /// Convert the temporary file into a `std::fs::File`.
    pub fn into_file(mut self) -> File {
        let NamedTempFileInner { path, file } = self.take_inner();
        let _ = fs::remove_file(path);
        file
    }
}

impl Drop for NamedTempFile {
    fn drop(&mut self) {
        if let Some(NamedTempFileInner { file, path }) = self.0.take() {
            drop(file);
            let _ = fs::remove_file(path);
        }
    }
}

impl Read for NamedTempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file_mut().read(buf)
    }
}

impl<'a> Read for &'a NamedTempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file().read(buf)
    }
}

impl Write for NamedTempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file_mut().write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.file_mut().flush()
    }
}

impl<'a> Write for &'a NamedTempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file().write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.file().flush()
    }
}

impl Seek for NamedTempFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file_mut().seek(pos)
    }
}

impl<'a> Seek for &'a NamedTempFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file().seek(pos)
    }
}

#[cfg(unix)]
impl std::os::unix::io::AsRawFd for NamedTempFile {
    #[inline]
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.file().as_raw_fd()
    }
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for NamedTempFile {
    #[inline]
    fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
        self.file().as_raw_handle()
    }
}


/// Create a new temporary file with custom parameters.
///
/// # Security
/// 
/// This variant is *NOT* secure/reliable in the presence of a pathological temporary file cleaner.
///
/// # Resource Leaking
/// 
/// `NamedTempFile`s are deleted on drop. As rust doesn't guarantee that a struct will ever be
/// dropped, these temporary files will not be deleted on abort, resource leak, early exit, etc.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NamedTempFileBuilder<'a, 'b> {
    random_len: usize,
    prefix: &'a str,
    suffix: &'b str,
}

impl<'a, 'b> NamedTempFileBuilder<'a, 'b> {
    /// Create a new `NamedTempFileBuilder`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # extern crate tempfile;
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// use tempfile::NamedTempFileBuilder;
    ///
    /// let named_temp_file = NamedTempFileBuilder::new()
    ///     .prefix("my-temporary-note")
    ///     .suffix(".txt")
    ///     .rand_bytes(5)
    ///     .create()?;
    /// 
    /// let name = named_temp_file
    ///     .path()
    ///     .file_name()?
    ///     .to_str()?;
    ///
    /// assert!(name.starts_with("my-temporary-note"));
    /// assert!(name.ends_with(".txt"));
    /// assert_eq!(name.len(), "my-temporary-note.txt".len() + 5);
    /// # }
    /// ```
    pub fn new() -> Self {
        NamedTempFileBuilder {
            random_len: ::NUM_RAND_CHARS,
            prefix: ".tmp",
            suffix: "",
        }
    }

    /// Set a custom filename prefix.
    ///
    /// Path separators are legal but not advisable.
    /// Default: `.tmp`.
    /// 
    /// # Examples
    /// 
    /// > TODO
    pub fn prefix(&mut self, prefix: &'a str) -> &mut Self {
        self.prefix = prefix;
        self
    }

    /// Set a custom filename suffix.
    ///
    /// Path separators are legal but not advisable.
    /// Default: empty.
    /// 
    /// # Examples
    /// 
    /// > TODO
    pub fn suffix(&mut self, suffix: &'b str) -> &mut Self {
        self.suffix = suffix;
        self
    }

    /// Set the number of random bytes.
    ///
    /// Default: `6`.
    /// 
    /// # Examples
    /// 
    /// > TODO
    pub fn rand_bytes(&mut self, rand: usize) -> &mut Self {
        self.random_len = rand;
        self
    }

    /// Create the named temporary file.
    /// 
    /// # Security
    /// 
    /// See: [`NamedTempFile::new`]
    ///  
    /// # Errors
    /// 
    /// > TODO
    /// 
    /// # Examples
    /// 
    /// > TODO
    /// 
    /// [`NamedTempFile::new`]: struct.NamedTempFile.html#method.new
    pub fn create(&self) -> io::Result<NamedTempFile> {
        self.create_in(&env::temp_dir())
    }

    /// Create the named temporary file in the specified directory.
    /// 
    /// # Errors
    /// 
    /// > TODO
    /// 
    /// # Examples
    /// 
    /// > TODO
    /// 
    /// [`NamedTempFile::new`]: struct.NamedTempFile.html#method.new
    pub fn create_in<P: AsRef<Path>>(&self, dir: P) -> io::Result<NamedTempFile> {
        for _ in 0..::NUM_RETRIES {
            let path = dir.as_ref().join(util::tmpname(self.prefix, self.suffix, self.random_len));
            return match imp::create_named(&path) {
                Ok(file) => {
                    Ok(NamedTempFile(Some(NamedTempFileInner {
                        path: path,
                        file: file,
                    })))
                }
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => Err(e),
            };
        }
        Err(io::Error::new(io::ErrorKind::AlreadyExists,
                           "too many temporary files exist"))

    }
}
