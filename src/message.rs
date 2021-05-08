use colored::*;

pub struct Message;
impl Message {
    pub fn error(message: &str) {
        println!("{} {}", "error:".bold().red(), message);
    }

    pub fn warning(message: &str) {
        println!("{} {}", "warning:".bold().yellow(), message);
    }

    pub fn hint(message: &str) {
        println!("{} {}", "hint:".bold().yellow(), message);
    }

    pub fn example(message: &str) {
        println!("\n{}\n{}\n", "example:".bold().yellow(), message);
    }
}
