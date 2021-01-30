pub mod client;
pub mod language;
pub mod data;
pub mod error;
mod deserialize;

mod protocol {
    tonic::include_proto!("nlpewee");
}

pub use client::{ClientBuilder, Client};
pub use data::*;
pub use language::Language;
