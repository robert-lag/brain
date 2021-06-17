use colored::*;
use indoc::indoc;

pub struct Message;
impl Message {
    pub fn display_correct_note_format() {
        Message::hint(indoc! {r##"
            the format of a note should look like this:

            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
            ---

            name: example-name
            date: yyyy-mm-dd HH:MM:SS
            tags: [ tag1, #tag2, tag-3 ]
            backlinks: [ ]

            ---

            custom note content in markdown (it doesn't matter what is written here)

            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

        "##});
    }

    pub fn error(message: &str) {
        println!("{} {}", "error:".bold().red(), message);
    }

    pub fn warning(message: &str) {
        println!("{} {}", "warning:".bold().yellow(), message);
    }

    pub fn hint(message: &str) {
        println!("{} {}", "hint:".bold().yellow(), message);
    }

    pub fn info(message: &str) {
        println!("{} {}", "info:".bold().blue(), message);
    }

    pub fn example(message: &str) {
        println!("\n{}\n{}\n", "example:".bold().yellow(), message);
    }
}
