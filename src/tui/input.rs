/// Reusable struct to implement the behaviour for the input field.
#[derive(Default, Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let new_input = Input::default();
        assert_eq!(new_input.to_string(), "");
    }

    #[test]
    fn test_put() {
        let mut input = Input::default();
        input.put(String::from("a"));
        assert_eq!(input.to_string(), "a");
        input.put(String::from("b"));
        assert_eq!(input.to_string(), "ab");
    }

    #[test]
    fn test_delete() {
        let mut input = Input::default();
        input.put(String::from("a"));
        input.put(String::from("b"));
        input.delete();
        assert_eq!(input.to_string(), "a");
    }
}
