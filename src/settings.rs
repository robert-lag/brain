use crate::history::History;
use crate::message::Message;

use std::ffi::OsString;

pub struct Settings {
    pub notes_dir: OsString,
    pub zettelkasten_dir: OsString,
    pub backlinking_enabled: bool,
    pub print_to_stdout: bool,
    pub show_interactive_dialogs: bool,
    pub note_history: History,
}

impl Settings {
    pub fn init(notes_dir: OsString, zettelkasten_dir: OsString) -> Self {
        let mut settings = Settings {
            notes_dir,
            zettelkasten_dir,
            note_history: History::new(),
            backlinking_enabled: true,
            print_to_stdout: true,
            show_interactive_dialogs: true,
        };

        if let Err(error) = settings
            .note_history
            .init(settings.zettelkasten_dir.as_os_str())
        {
            Message::info(&("initializing history: ".to_string() + &error));
        }
        settings
    }
}
