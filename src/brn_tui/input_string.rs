pub struct InputString {
    text: String,
    pre_text_length: usize,
}

impl InputString {
    pub fn from(pre_text: &str) -> InputString {
        InputString {
            text: String::from(pre_text),
            pre_text_length: pre_text.len(),
        }
    }

    pub fn push(&mut self, c: char) {
        self.text.push(c);
    }

    pub fn pop(&mut self) {
        if self.text.len() > self.pre_text_length {
            self.text.pop();
        }
    }

    pub fn get_displayed_text(&self) -> String {
        return self.text.clone();
    }

    pub fn get_content_text(&self) -> String {
        return self
            .text
            .chars()
            .into_iter()
            .skip(self.pre_text_length)
            .collect();
    }

    pub fn set_pre_text(&mut self, pre_text: &str) {
        self.text = String::from(pre_text);
        self.pre_text_length = pre_text.len();
    }

    pub fn clear(&mut self) {
        while self.text.len() > self.pre_text_length {
            self.text.pop();
        }
    }
}
