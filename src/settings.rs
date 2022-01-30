use crate::history::History;
use crate::message::Message;

use std::ffi::OsString;
use std::path::PathBuf;

pub struct Settings {
    pub notes_dir: OsString,
    pub zettelkasten_dir: OsString,
    pub backlinking_enabled: bool,
    pub print_to_stdout: bool,
    pub note_history: History,
}

impl Settings {
    pub fn init(notes_dir: OsString, zettelkasten_dir: OsString) -> Self {
        let mut settings = Settings {
            notes_dir: notes_dir,
            zettelkasten_dir: zettelkasten_dir,
            note_history: History::new(),
            backlinking_enabled: true,
            print_to_stdout: true,
        };

        let history_file_path = PathBuf::from(&settings.zettelkasten_dir).join("history");
        if let Err(error) = settings.note_history.init(history_file_path) {
            Message::error(&("Initializing history: ".to_string() + &error));
        }
        return settings;
    }
}
