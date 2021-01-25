mod parse;

pub use parse::{parse_group, Robot};

#[cfg(feature = "twitter")]
pub mod twitter;
