use tui::widgets::ListState;

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> Default for StatefulList<T> {
    fn default() -> StatefulList<T> {
        StatefulList::with_items(Vec::new())
    }
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn replace_items_with(&mut self, items: Vec<T>) {
        self.items = items;
        self.state = ListState::default();
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.state.select(index);
    }

    pub fn selected(&mut self) -> Option<usize> {
        self.state.selected()
    }

    pub fn selected_item(&mut self) -> Option<&T> {
        if self.items.is_empty() {
            return None;
        }

        if let Some(selected_index) = self.selected() {
            Some(&self.items[selected_index])
        } else {
            None
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let next_index = match self.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    self.items.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(next_index));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let prev_index = match self.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(prev_index));
    }

    pub fn get_items(&mut self) -> &mut Vec<T> {
        &mut self.items
    }

    pub fn get_state(&mut self) -> &mut ListState {
        &mut self.state
    }
}
