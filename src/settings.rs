pub struct Settings {
    pub notes_dir: String
}

impl Settings {
    pub fn new() -> Self {
        return Settings {
            notes_dir: String::new()
        }
    }
}
