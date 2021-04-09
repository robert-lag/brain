use colored::*;
use std::io::Error;

pub struct Message;
impl Message {
    pub fn error(message: &str) {
        println!("{} {}", "error:".bold().red(), message);
    }

    pub fn error_from(error: Error) {
        println!("{} {}", "error:".bold().red(), error);
    }

    pub fn warning(message: &str) {
        println!("{} {}", "warning:".bold().yellow(), message);
    }

    pub fn example(message: &str) {
        println!("\n{}\n{}\n", "example:".bold().yellow(), message);
    }
}
