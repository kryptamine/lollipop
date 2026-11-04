//! Advice read aloud. The voice follows the Advice Widget and never queues.

mod reader;
mod speech;
mod spoken;

pub use reader::Reader;
pub use speech::Speech;
