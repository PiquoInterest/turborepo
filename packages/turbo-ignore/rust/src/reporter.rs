use crate::sanitize_for_log;

const PREFIX: &str = "≫  ";

pub trait Reporter: Send + Sync {
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
    fn log(&self, message: &str);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ConsoleReporter;

impl Reporter for ConsoleReporter {
    fn info(&self, message: &str) {
        println!("{PREFIX}{}", sanitize_for_log(message));
    }

    fn warn(&self, message: &str) {
        eprintln!("{PREFIX}{}", sanitize_for_log(message));
    }

    fn error(&self, message: &str) {
        eprintln!("{PREFIX}{}", sanitize_for_log(message));
    }

    fn log(&self, message: &str) {
        println!("{}", sanitize_for_log(message));
    }
}
