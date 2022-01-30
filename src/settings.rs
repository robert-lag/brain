use crate::history::History;

use std::ffi::OsString;

pub struct Settings {
    pub notes_dir: OsString,
    pub zettelkasten_dir: OsString,
    pub backlinking_enabled: bool,
    pub print_to_stdout: bool,
    pub note_history: History,
}

impl Settings {
    pub fn new() -> Self {
        return Settings {
            notes_dir: OsString::new(),
            zettelkasten_dir: OsString::new(),
            note_history: History::init(),
            backlinking_enabled: true,
            print_to_stdout: true,
        };
    }
}
