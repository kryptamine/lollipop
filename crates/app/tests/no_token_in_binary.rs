//! The built app, scanned for what a GitHub token looks like.
#![allow(clippy::unwrap_used, reason = "a failed test should panic")]

// GitHub's prefixes for a fine-grained and a classic token, and the shortest body of each.
const TOKEN_SHAPES: [(&[u8], usize); 2] = [(b"github_pat_", 22), (b"ghp_", 36)];

fn looks_like_a_token(binary: &[u8], prefix: &[u8], body: usize) -> bool {
    binary.windows(prefix.len() + body).any(|window| {
        let (start, rest) = window.split_at(prefix.len());
        start == prefix
            && rest
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    })
}

// The private phase's token reads a private Release, and a binary can be copied and read.
#[test]
fn the_binary_holds_no_github_token() {
    let binary = std::fs::read(env!("CARGO_BIN_EXE_lollipop")).unwrap();
    for (prefix, body) in TOKEN_SHAPES {
        assert!(
            !looks_like_a_token(&binary, prefix, body),
            "{}",
            String::from_utf8_lossy(prefix)
        );
    }
}
