//! Errors returned by this library

#[derive(Debug)]
pub enum Error {
    NoHost,
    InvalidHost,
    InvalidEmail,
    InvalidDomain(String),
}
