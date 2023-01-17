use const_format::formatcp;
use regex::Regex;

const ESCAPE_CHAR: char = 27 as char;
const REGEX: &str = formatcp!("{}\\[(?:2K|1G)", ESCAPE_CHAR);

pub fn try_cleanup(input: &str) -> String {
    let cleaned = input.trim_end();
    if cleaned.is_empty() {
        String::new()
    } else {
        let regex = Regex::new(REGEX).unwrap();
        format!("{}\n", regex.replace_all(cleaned, ""))
    }
}
