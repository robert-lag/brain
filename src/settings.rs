use std::ffi::OsString;

pub struct Settings {
    pub notes_dir: OsString,
    pub zettelkasten_dir: OsString
}

impl Settings {
    pub fn new() -> Self {
        return Settings {
            notes_dir: OsString::new(),
            zettelkasten_dir: OsString::new()
        }
    }
}
