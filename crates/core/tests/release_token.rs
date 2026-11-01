//! The token as `security` reads it from the Keychain, with `security` faked on `PATH`.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::os::unix::fs::PermissionsExt;

use lollipop_core::delivery::ReleaseToken;
use tempfile::TempDir;

use support::delivery_bench::{table_with_cs, until, DeliveryBench};
use support::fake_release::TOKEN;
use support::staged_game::Stage;

// Exit 44 is what the real tool gives when the Keychain has no such item.
fn fake_security(answer: &str) -> String {
    format!(
        "#!/bin/sh\n\
         [ \"$*\" = 'find-generic-password -s lollipop-benchmark-token -w' ] || exit 1\n\
         {answer}\n"
    )
}

// One test, as `PATH` belongs to the whole process.
#[tokio::test]
async fn the_token_comes_from_the_keychain_at_run_time() {
    let bin = TempDir::new().unwrap();
    let security = bin.path().join("security");
    std::fs::write(&security, fake_security("exit 44")).unwrap();
    std::fs::set_permissions(&security, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("PATH", bin.path());

    assert!(ReleaseToken::from_keychain().is_none());

    std::fs::write(&security, fake_security(&format!("echo {TOKEN}"))).unwrap();
    let token = ReleaseToken::from_keychain().unwrap();

    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let table = table_with_cs(100.0);
    bench.release.publish(1, &table);
    bench.start_polling();
    bench.start_downloads_on(bench.release.channel_with(token));
    until("the table is live", || {
        bench.live().as_ref() == Some(&table)
    })
    .await;
}
