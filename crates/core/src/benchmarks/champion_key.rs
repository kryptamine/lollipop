/// Match-V5 writes `MasterYi` and `Kaisa` where Live Client Data writes `Master Yi` and
/// `Kai'Sa`. These three names differ by more than spelling, and an unlisted one only
/// costs its Markers.
const RENAMED: [(&str, &str); 3] = [
    ("wukong", "monkeyking"),
    ("nunuwillump", "nunu"),
    ("renataglasc", "renata"),
];

/// What a champion's name from either API reduces to, so the two can be compared.
pub(super) fn champion_key(name: &str) -> String {
    let key: String = name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    match RENAMED.iter().find(|(display, _)| *display == key) {
        Some((_, id)) => (*id).to_string(),
        None => key,
    }
}
