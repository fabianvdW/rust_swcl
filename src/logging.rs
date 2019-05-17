use std::fs::OpenOptions;
use std::sync::Mutex;
use std::fs::File;
use std::io::Write;

pub struct Logger {
    pub file: Mutex<File>,
}

impl Logger {
    pub fn new(path: &str, append: bool) -> Self {
        if !append {
            match std::fs::remove_file(path) {
                _ => {}
            }
        };
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .open(path)
            .unwrap();
        Logger {
            file: Mutex::new(file),
        }
    }

    pub fn log(&self, msg: &str, also_stdout: bool) {
        self.file
            .lock()
            .unwrap()
            .write(msg.as_bytes())
            .expect("Something went wrong when trying to write to this file");
        if also_stdout {
            print!("{}", msg);
        }
    }
}