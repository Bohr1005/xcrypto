/*
 * @Author: Bohr shiyu.he@jyquant.com.cn
 * @Date: 2023-06-25 15:39:43
 * @LastEditTime: 2023-09-26 15:15:26
 * @FilePath: lib.rs
 */
use chrono::{DateTime, Local};
use log::{LevelFilter, Metadata, Record};
use std::{
    io::{BufWriter, Write},
    thread::JoinHandle,
};

pub use log::{debug, error, info, trace, warn};
pub type Level = LevelFilter;

#[derive(Debug)]
struct LogEntry {
    datetime: DateTime<Local>,
    level: log::Level,
    file: String,
    line: u32,
    msg: String,
}

impl LogEntry {
    pub fn datetime(&self) -> &DateTime<Local> {
        &self.datetime
    }
    pub fn level(&self) -> &log::Level {
        &self.level
    }
    pub fn file(&self) -> &str {
        &self.file
    }
    pub fn line(&self) -> u32 {
        self.line
    }
    pub fn msg(&self) -> &str {
        &self.msg
    }
}
enum Action {
    Write(LogEntry),
    Flush,
    Exit,
}

#[derive(Debug)]
struct Context<P: ToString + Send> {
    rx: crossbeam_channel::Receiver<Action>,
    path: Option<P>,
    date: chrono::NaiveDate,
}

pub struct Handle {
    tx: crossbeam_channel::Sender<Action>,
    thread: Option<JoinHandle<()>>,
}

impl Handle {
    pub fn stop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = self.tx.send(Action::Exit);
            let _ = thread.join();
        }
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        self.stop();
    }
}

struct Logger {
    tx: crossbeam_channel::Sender<Action>,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let entry = LogEntry {
            datetime: Local::now(),
            level: record.level(),
            file: record.file().unwrap_or("unknown").to_string(),
            line: record.line().unwrap_or(0),
            msg: record.args().to_string(),
        };

        let _ = self.tx.send(Action::Write(entry));
    }

    fn flush(&self) {
        let _ = self.tx.send(Action::Flush);
    }
}

fn open_file(path: &str) -> Result<std::fs::File, std::io::Error> {
    let dir = std::path::Path::new(path);
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
}

fn rotate<P: ToString + Send>(
    ctx: &Context<P>,
) -> Result<BufWriter<Box<dyn Write>>, std::io::Error> {
    match &ctx.path {
        Some(path) => {
            let postfix = ctx.date.format("_%Y%m%d.log").to_string();

            let path = path.to_string() + &postfix;
            let file = open_file(&path)?;
            Ok(BufWriter::new(Box::new(file)))
        }
        None => {
            let target = Box::new(std::io::stdout());
            Ok(BufWriter::new(target))
        }
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn worker<P: ToString + Send>(mut ctx: Context<P>) -> Result<(), std::io::Error> {
    let timeout = std::time::Duration::from_secs(1);

    let mut target = rotate(&ctx)?;
    let mut ts = now();
    loop {
        let today = Local::now().date_naive();

        if today != ctx.date {
            ctx.date = today;
            target = rotate(&ctx)?;
        }

        if let Ok(action) = ctx.rx.recv_timeout(timeout) {
            match action {
                Action::Write(entry) => {
                    let time = entry.datetime().time();
                    let filename = entry.file();
                    let filename = match filename.rfind("/") {
                        Some(index) => filename.get(index + 1..).unwrap_or("unknown"),
                        None => filename,
                    };

                    let line = entry.line();
                    let level = entry.level();
                    let msg = entry.msg();

                    target.write_all(
                        format!("[{} {} {} {}] {}\n", time, filename, line, level, msg).as_bytes(),
                    )?;
                }
                Action::Flush => {
                    target.flush()?;
                }
                Action::Exit => {
                    target.flush()?;
                    break;
                }
            }
        }

        let n = now();
        if n - ts >= 1 {
            ts = n;
            target.flush()?;
        }
    }

    Ok(())
}

pub fn init<P: ToString + Send + 'static>(path: Option<P>, level: Level) -> Handle {
    let (tx, rx) = crossbeam_channel::unbounded();

    let ctx = Context {
        rx,
        path,
        date: Local::now().date_naive(),
    };

    let logger = Logger { tx: tx.clone() };

    log::set_boxed_logger(Box::new(logger)).expect("error to init logger");
    log::set_max_level(level);

    let thread = std::thread::spawn(move || {
        if let Err(msg) = worker(ctx) {
            eprintln!("error {}", msg);
        }
    });

    Handle {
        tx,
        thread: Some(thread),
    }
}
