#[derive(Default, Clone)]
pub struct Input {
    val: String,
}

impl Input {
    pub fn put(&mut self, char: String) {
        self.val.push_str(&char);
    }

    pub fn delete(&mut self) {
        self.val.truncate(self.val.len().saturating_sub(1));
    }
}

impl ToString for Input {
    fn to_string(&self) -> String {
        self.val.to_string()
    }
}
