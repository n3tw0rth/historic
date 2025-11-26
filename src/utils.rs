use rust_fuzzy_search::fuzzy_search_best_n;

use crate::Result;

pub fn string_to_md5(strings: &str) -> String {
    let cleaned_str = strings.replace(" ", "");
    let digest = md5::compute(&cleaned_str);
    format!("{:x}", digest)
}

pub fn fuzzy_search(source: Vec<String>, search: &str) -> Result<Vec<String>> {
    fuzzy_search(source, search)
}
