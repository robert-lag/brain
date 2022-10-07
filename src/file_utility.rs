use std::fs::File;
use std::io::{Error, Read};
use std::path::Path;

pub struct FileUtility;

impl FileUtility {
    pub fn get_content_from_file<P: AsRef<Path>>(path: P) -> Result<String, Error> {
        let mut file = match File::open(path) {
            Ok(opened_file) => opened_file,
            Err(error) => return Err(error),
        };
        let mut file_content = String::new();

        match file.read_to_string(&mut file_content) {
            Ok(_) => {}
            Err(error) => return Err(error),
        };

        return Ok(file_content);
    }
}
