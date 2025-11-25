#[derive(Default, Clone)]
pub struct Input {
    pub val: String,
}

impl Input {
    pub fn put(&mut self, char: String) {
        self.val.push_str(&char);
    }

    pub fn delete(&mut self) {
        self.val.truncate(self.val.len() - 1);
    }
}
