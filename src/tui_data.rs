use tui::widgets::ListState;

pub struct TuiData {
    pub note_list: ListState,
    pub note_list_data: Vec<String>,
    pub note_content_preview: String,
}

impl Default for TuiData {
    fn default() -> TuiData {
        let mut tui_data = TuiData {
            note_list: ListState::default(),
            note_list_data: Vec::new(),
            note_content_preview: String::default(),
        };
        tui_data.note_list.select(Some(0));
        return tui_data;
    }
}

impl TuiData {

}