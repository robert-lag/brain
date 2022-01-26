pub struct InputString {
    text: String,
}

impl Default for InputString {
    fn default() -> InputString {
        InputString {
            text: String::from("/"),
        }
    }
}

impl InputString {
    pub fn push(&mut self, c: char) {
        self.text.push(c);
    }

    pub fn pop(&mut self) {
        if self.text.len() > 1 {
            self.text.pop();
        }
    }

    pub fn get_displayed_text(&self) -> String {
        return self.text.clone();
    }

    pub fn get_content_text(&self) -> String {
        return self.text.chars().into_iter().skip(1).collect();
    }
}