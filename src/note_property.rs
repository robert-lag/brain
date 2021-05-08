use std::fmt;

#[derive(Debug)]
pub enum NoteProperty {
    NoteId,
    NoteName,
    FileName,
    CreationDate
}

impl NoteProperty {
    pub fn to_db_string(&self) -> String {
        match self {
            NoteProperty::NoteId => "note_id".to_string(),
            NoteProperty::NoteName => "note_name".to_string(),
            NoteProperty::FileName => "file_name".to_string(),
            NoteProperty::CreationDate => "creation_date".to_string(),
        }
    }
}

impl fmt::Display for NoteProperty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
