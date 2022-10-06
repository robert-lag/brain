use crate::message::Message;
use std::ffi::OsStr;
use std::path::Path;

pub struct Directory;
impl Directory {
    pub fn is_zettelkasten_dir(directory: &OsStr, hide_error_messages: bool) -> bool {
        if Path::new(directory).join(".zettelkasten").exists() {
            return true;
        } else {
            if !hide_error_messages {
                Message::error(&format!(
                    "the specified path is not zettelkasten directory: '{}'",
                    directory.to_string_lossy()
                ));
                Message::hint("use the 'init' subcommand to initialize a zettelkasten directory");
            }
            return false;
        }
    }
}
