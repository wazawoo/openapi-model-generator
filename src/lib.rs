pub mod cli;
pub mod error;
pub mod generator;
pub mod models;
pub mod parser;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
