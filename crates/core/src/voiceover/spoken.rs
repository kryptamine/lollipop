/// `text` as the voice should hear it. Apple's voices name a symbol aloud rather than
/// pause on it, so a symbol that only marks a pause becomes a comma and markup is dropped.
pub(super) fn spoken(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // A clock time, as in 12:30, is read as one.
            ':' if out.ends_with(|c: char| c.is_ascii_digit())
                && chars.peek().is_some_and(char::is_ascii_digit) =>
            {
                out.push(c);
            }
            ',' | ':' | ';' | '—' | '–' | '→' | '←' | '|' => pause(&mut out),
            '-' if chars.peek() == Some(&'>') => {
                chars.next();
                pause(&mut out);
            }
            // A hyphen inside a word, as in «кто-то», or a minus before a number, stays.
            '-' if !out.ends_with(char::is_alphanumeric)
                && !chars.peek().is_some_and(char::is_ascii_digit) =>
            {
                pause(&mut out);
            }
            '(' | ')' | '[' | ']' | '{' | '}' | '"' | '«' | '»' | '“' | '”' | '„' | '*' | '#'
            | '_' | '`' | '/' | '\\' => space(&mut out),
            '.' | '!' | '?' => {
                out.truncate(out.trim_end_matches([',', ' ']).len());
                out.push(c);
            }
            c if c.is_whitespace() => space(&mut out),
            c => {
                if out.ends_with(',') {
                    out.push(' ');
                }
                out.push(c);
            }
        }
    }
    out.trim_matches([',', ' ']).to_string()
}

/// A comma, unless one or a sentence end already precedes it.
fn pause(out: &mut String) {
    out.truncate(out.trim_end().len());
    if !out.ends_with([',', '.', '!', '?']) {
        out.push(',');
    }
}

fn space(out: &mut String) {
    if !out.is_empty() && !out.ends_with([' ', ',']) {
        out.push(' ');
    }
}
