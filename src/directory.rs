use crate::message::Message;
use std::ffi::OsStr;
use std::fs;
use std::path::{ Path, PathBuf };
use walkdir::WalkDir;

pub struct Directory;
impl Directory {
    pub fn is_zettelkasten_dir(directory: &OsStr, hide_error_messages: bool) -> bool {
        if Path::new(directory).join(".zettelkasten").exists() {
            return true;
        } else {
            if !hide_error_messages {
                Message::error(&format!("the specified path is not zettelkasten directory: '{}'", directory.to_string_lossy()));
                Message::hint("use the 'init' subcommand to initialize a zettelkasten directory");
            }
            return false;
        }
    }

    pub fn copy(source_dir: &OsStr, target_dir: &OsStr) {
        for entry in WalkDir::new(source_dir) {
            let entry = match entry {
                Ok(value) => value,
                Err(error) => {
                    Message::error(&format!("couldn't iterate through directory '{}': '{}'", source_dir.to_string_lossy(), error));
                    return;
                }
            };
            let file_name = entry.file_name();
            let target_path = Path::new(target_dir).join(file_name);
            let source_path = entry.path().as_os_str();

            let is_root_directory: bool = entry.path().canonicalize().unwrap() == PathBuf::from(source_dir).canonicalize().unwrap();
            if is_root_directory {
                continue;
            }

            let copy_result = fs::copy(&source_path, &target_path);
            match copy_result {
                Ok(_) => {  },
                Err(error) => Message::error(&format!("couldn't clone from {} to {}:\n'{}'\n",
                    source_path.to_string_lossy(),
                    target_path.to_string_lossy(),
                    error)),
            }
        }
    }
}
