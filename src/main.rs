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

fn metadata_modified(filepath: &str) -> Result<SystemTime, Error> {
    fs::metadata(filepath)
        .map_err(|err| Error::new(format!("failed to read metadata from file {err}")))?
        .modified()
        .map_err(|err| Error::new(format!("failed to read modified time from metadata {err}",)))
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(Error::new("provide filepath and command"));
    }
    let filepath = &args[1];
    let command = &args[2];
    if !Path::new(filepath).exists() {
        return Err(Error::new("file does not exist"));
    }
    let mut last_modified_time = metadata_modified(filepath)?;
    println!("watching");
    loop {
        sleep(Duration::from_millis(1));
        let time = metadata_modified(filepath)?;
        if time > last_modified_time {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .map_err(|err| Error::new(format!("failed to execute command {err}")))?;
            println!("{}", String::from_utf8_lossy(&output.stdout));
            last_modified_time = time;
        }
    }
}
