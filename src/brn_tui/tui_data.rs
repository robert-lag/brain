use crate::notes::Notes;
use crate::brn_tui::stateful_list::StatefulList;
use crate::brn_tui::input_mode::InputMode;
use crate::brn_tui::input_string::InputString;

pub struct TuiData {
    pub note_list: StatefulList<String>,
    pub note_content_preview: String,
    pub message: String,
    pub search_text: InputString,
    pub input_mode: InputMode,
}

impl Default for TuiData {
    fn default() -> TuiData {
        let mut tui_data = TuiData {
            note_list: StatefulList::with_items(Notes::get(100)),
            note_content_preview: String::default(),
            message: String::default(),
            search_text: InputString::default(),
            input_mode: InputMode::Normal,
        };
        tui_data.note_list.select(Some(0));
        return tui_data;
    }
}

impl TuiData {

}