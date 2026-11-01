//! Brings the table the manifest names to the app, from a private GitHub Release.

mod keep_table_current;
mod manifest;
mod manifest_key;
mod release_channel;
mod release_token;
mod signed_message;
mod table_file;

pub use keep_table_current::keep_table_current;
pub use manifest_key::ManifestKey;
pub use release_channel::{ReleaseChannel, GITHUB_API, RELEASE_REPO};
pub use release_token::ReleaseToken;
pub use signed_message::signed_message;
pub use table_file::TableFile;
