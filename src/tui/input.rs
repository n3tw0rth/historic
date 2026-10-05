/// Reusable struct to implement the behaviour for the input field.
#[derive(Default, Debug, Clone)]
pub struct Input {
    val: String,
}

impl Input {
    pub fn put(&mut self, char: String) {
        self.val.push_str(&char);
    }

    /// Remove the last character.
    pub fn delete(&mut self) {
        self.val.pop();
    }

    /// Remove the last word along with any whitespace after it.
    pub fn delete_word(&mut self) {
        let trimmed = self.val.trim_end();
        let start = trimmed
            .char_indices()
            .rev()
            .find(|(_, c)| c.is_whitespace())
            .map_or(0, |(i, c)| i + c.len_utf8());
        self.val.truncate(start);
    }

    pub fn clear(&mut self) {
        self.val.clear();
    }

    pub fn value(&self) -> &str {
        &self.val
    }

    pub fn is_empty(&self) -> bool {
        self.val.is_empty()
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.val)
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

    #[test]
    fn test_delete_multibyte() {
        let mut input = Input::default();
        input.put(String::from("caf"));
        input.put(String::from("é"));
        input.delete();
        assert_eq!(input.to_string(), "caf");
    }

    #[test]
    fn test_delete_word() {
        let mut input = Input::default();
        input.put(String::from("git commit  "));
        input.delete_word();
        assert_eq!(input.to_string(), "git ");
        input.delete_word();
        assert_eq!(input.to_string(), "");
        input.delete_word();
        assert_eq!(input.to_string(), "");
    }

    #[test]
    fn test_clear() {
        let mut input = Input::default();
        input.put(String::from("abc"));
        input.clear();
        assert!(input.is_empty());
    }
}
