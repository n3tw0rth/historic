pub fn string_to_md5(strings: &String) -> String {
    let cleaned_str = strings.replace(" ", "");
    let digest = md5::compute(&cleaned_str);
    format!("{:x}", digest)
}
