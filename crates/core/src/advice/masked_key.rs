/// The only form of a key the view ever gets: its last four characters.
#[must_use]
pub fn masked_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    let last_four: String = chars.iter().skip(chars.len().saturating_sub(4)).collect();
    format!("sk-…{last_four}")
}
