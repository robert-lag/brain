use crate::note_utility::NoteUtility;
use crate::brn_tui::stateful_list::StatefulList;
use crate::brn_tui::input_mode::InputMode;
use crate::brn_tui::input_string::InputString;

pub struct TuiData {
    pub note_list: StatefulList<String>,
    pub note_content_preview: String,
    pub message: String,
    pub search_text: InputString,
    pub edit_text: InputString,
    pub input_mode: InputMode,
    pub note_name_cache: String,
    pub note_list_title: String,
}

impl Default for TuiData {
    fn default() -> TuiData {
        let mut tui_data = TuiData {
            note_list: StatefulList::with_items(NoteUtility::get(100)),
            note_content_preview: String::default(),
            message: String::default(),
            search_text: InputString::from("/"),
            edit_text: InputString::from("Name: "),
            input_mode: InputMode::Normal,
            note_name_cache: String::default(),
            note_list_title: String::from("List"),
        };
        tui_data.note_list.select(Some(0));
        return tui_data;
    }
}

impl TuiData {

}