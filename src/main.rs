use std::collections::hash_map::DefaultHasher;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, SystemTime};
use std::{error, fmt, fs};
use tokio::time;

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(Error::new("try `watcher 'file1 file2' 'cmd'`"));
    }
    let paths: Vec<&str> = args[1].split_whitespace().collect();
    let cmd = args[2].clone();
    let cmd = Arc::new(Mutex::new(cmd));
    for path in &paths {
        if !Path::new(path).exists() {
            return Err(Error::new(format!("file does not exist at {path}")));
        }
    }
    let (tx, rx) = mpsc::channel();
    let rx = Arc::new(Mutex::new(rx));
    let h = tokio::spawn(async {
        println!("started");
        time::sleep(time::Duration::from_secs(3)).await;
        println!("ended");
    });
    let abort = h.abort_handle();
    let mut handle: Option<tokio::task::JoinHandle<()>> = Some(h);
    let is_running = Arc::new(AtomicBool::new(true));
    // thread::spawn(move || {
    //     let mut i = 0;
    //     let rx = rx.clone();
    //     while let Ok(_) = rx.lock().expect("failed to lock").recv() {
    //         println!("i={i}");
    //         i += 1;
    //         if exec(&cmd).is_err() {
    //             break;
    //         }
    //     }
    // });
    let mut prev_hash = hash(&paths)?;
    let mut i = 0;
    loop {
        println!("loop");
        sleep(Duration::from_millis(500));
        // abort.abort();
        let tx = tx.clone();
        let rx = rx.clone();
        let cmd = cmd.clone();
        let last_hash = hash(&paths)?;
        if prev_hash != last_hash {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("i={i}");
            i += 1;
            if let Some(handle) = handle.take() {
                println!("joining");
                is_running.store(false, Ordering::Relaxed);
                // handle.join().expect("failed to join handle");
            }
            println!("before jh");
            is_running.store(true, Ordering::Relaxed);
            println!("restarted is running");
            let is_running = is_running.clone();
            let jh = thread::spawn(move || {
                println!("SPAWN");
                if is_running.load(Ordering::Relaxed) {
                    println!("is running");
                    if let Ok(_) = rx.lock().expect("failed to lock rx").recv()
                    // cmd.lock().expect("failed to lock cmd").clone(),
                    {
                        if exec(&*cmd.lock().expect("failed to lock cmd")).is_err() {
                            return;
                        }
                        println!("Task executed");
                        // if exec(&cmd).is_err() {
                        //     break;
                        // }
                    }
                }
                println!("after `while is_running`");
                // let cmd = cmd.clone();
            });
            // handle = Some(jh);
            println!("after jh");
            if let Err(err) = tx.send(()) {
                return Err(Error::new(format!("failed to send: '{err}'")));
            }
            println!("end");
        }
        prev_hash = last_hash;
    }
    // println!("after the loop");
}
