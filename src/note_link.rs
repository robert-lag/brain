pub struct NoteLink {
    pub source_note_id: String,
    pub target_note_id: String,
}

impl NoteLink {
    pub fn new(source_note_id: String, target_note_id: String) -> Self {
        NoteLink {
            source_note_id,
            target_note_id
        }
    }
}