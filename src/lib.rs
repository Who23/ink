pub mod log;
pub mod diff;

use std::io;

#[derive(Debug)]
pub enum InkError {
    Err(&'static str),
    Uninitialized,
    Malformed(&'static str),
    IO(io::Error)
}

impl From<io::Error> for InkError {
    fn from(err: io::Error) -> InkError {
        InkError::IO(err)
    }
}