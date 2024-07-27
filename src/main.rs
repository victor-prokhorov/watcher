use std::collections::hash_map::DefaultHasher;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, sleep};
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
    let cmd = args[2].clone();
    for path in &paths {
        if !Path::new(path).exists() {
            return Err(Error::new(format!("file does not exist at {path}")));
        }
    }
    exec(&cmd)?;
    let (tx, rx) = mpsc::channel();
    let rx = Arc::new(Mutex::new(rx));
    thread::spawn(move || {
        let mut i = 0;
        let rx = rx.clone();
        while let Ok(_) = rx.lock().expect("failed to lock").recv() {
            println!("i={i}");
            i += 1;
            if exec(&cmd).is_err() {
                break;
            }
        }
    });
    let mut prev_hash = hash(&paths)?;
    loop {
        sleep(Duration::from_millis(100));
        let tx = tx.clone();
        let last_hash = hash(&paths)?;
        if prev_hash != last_hash {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            if let Err(err) = tx.send(()) {
                return Err(Error::new(format!("failed to send: '{err}'")));
            }
        }
        prev_hash = last_hash;
    }
    // println!("after the loop");
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
