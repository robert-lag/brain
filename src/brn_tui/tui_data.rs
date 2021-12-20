use crate::notes::Notes;
use crate::brn_tui::stateful_list::StatefulList;

pub struct TuiData {
    pub note_list: StatefulList<String>,
    pub note_content_preview: String,
}

impl Default for TuiData {
    fn default() -> TuiData {
        let mut tui_data = TuiData {
            note_list: StatefulList::with_items(Notes::get(100)),
            note_content_preview: String::default(),
        };
        tui_data.note_list.select(Some(0));
        return tui_data;
    }
}

impl TuiData {

}