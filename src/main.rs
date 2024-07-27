use std::collections::hash_map::DefaultHasher;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use std::{error, fmt, fs};

struct Error(String);

impl Error {
    fn new(message: impl Into<String>) -> Error {
        Error(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for Error {}

fn last_modified(filepath: &str) -> Result<SystemTime, Error> {
    fs::metadata(filepath)
        .map_err(|err| {
            Error::new(format!(
                "failed to read metadata from file at {filepath}: '{err}'"
            ))
        })?
        .modified()
        .map_err(|err| {
            Error::new(format!(
                "failed to read modified time from metadata at {filepath}: '{err}'",
            ))
        })
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(Error::new("try `watcher 'file1 file2' 'cmd'`"));
    }
    let paths: Vec<&str> = args[1].split_whitespace().collect();
    let cmd = &args[2];
    for path in &paths {
        if !Path::new(path).exists() {
            return Err(Error::new(format!("file does not exist at {path}")));
        }
    }
    exec(cmd)?;
    let mut prev_hash = hash(&paths)?;
    let mut i = 0;
    loop {
        sleep(Duration::from_millis(100));
        let last_hash = hash(&paths)?;
        if prev_hash != last_hash {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            exec(cmd)?;
            dbg!(i);
            i += 1;
        }
        prev_hash = last_hash;
    }
}

fn exec(cmd: impl AsRef<OsStr>) -> Result<(), Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|err| Error::new(format!("failed to execute command '{err}'")))?;
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

fn hash(paths: &[&str]) -> Result<u64, Error> {
    let mut hasher = DefaultHasher::new();
    for path in paths {
        last_modified(path)?.hash(&mut hasher);
    }
    Ok(hasher.finish())
}
