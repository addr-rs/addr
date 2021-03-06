/*!
  Robust domain name parsing using the Public Suffix List

  This library allows you to easily and accurately parse any given domain name.

  ## Examples

  ```rust
  # fn main() -> addr::Result<()> {
  use addr::{dns, domain};

  // You can find out the root domain
  // or extension of any given domain name
  let domain = domain::Name::parse("www.example.com")?;
  assert_eq!(domain.root(), Some("example.com"));
  assert_eq!(domain.suffix(), "com");

  let domain = domain::Name::parse("www.食狮.中国")?;
  assert_eq!(domain.root(), Some("食狮.中国"));
  assert_eq!(domain.suffix(), "中国");

  let domain = domain::Name::parse("www.xn--85x722f.xn--55qx5d.cn")?;
  assert_eq!(domain.root(), Some("xn--85x722f.xn--55qx5d.cn"));
  assert_eq!(domain.suffix(), "xn--55qx5d.cn");

  let domain = domain::Name::parse("a.b.example.uk.com")?;
  assert_eq!(domain.root(), Some("example.uk.com"));
  assert_eq!(domain.suffix(), "uk.com");

  let name = dns::Name::parse("_tcp.example.com.")?;
  assert_eq!(name.suffix(), Some("com."));

  // In any case if the domain's suffix is in the list
  // then this is definately a registrable domain name
  assert!(domain.has_known_suffix());
  # Ok(())
  # }
  ```
!*/

#![no_std]
#![forbid(unsafe_code)]

pub mod dns;
pub mod domain;
#[cfg(any(feature = "net", feature = "serde-net"))]
pub mod email;
mod matcher;
#[cfg(any(feature = "net", feature = "serde-net"))]
pub mod net;
#[cfg(any(feature = "serde", feature = "serde-net"))]
mod serde;

use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[non_exhaustive]
pub enum Error {
    NameTooLong,
    EmptyLabel,
    EmailLocalTooLong,
    EmailTooLong,
    EmptyName,
    IllegalCharacter,
    InvalidDomain,
    InvalidIpAddr,
    LabelEndNotAlnum,
    LabelStartNotAlnum,
    LabelTooLong,
    NoAtSign,
    NoHostPart,
    NoUserPart,
    NumericTld,
    QuoteUnclosed,
    TooManyLabels,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error = match self {
            Error::NameTooLong => "name too long",
            Error::EmptyLabel => "domain/email contains empty label",
            Error::EmailLocalTooLong => "email local too long",
            Error::EmailTooLong => "email too long",
            Error::EmptyName => "name is empty",
            Error::IllegalCharacter => "domain contains illegal characters",
            Error::InvalidDomain => "invalid domain name",
            Error::InvalidIpAddr => "email has an invalid ip address",
            Error::LabelEndNotAlnum => "label does not start with an alphanumeric character",
            Error::LabelStartNotAlnum => "label does not end with a alphanumeric character",
            Error::LabelTooLong => "label too long",
            Error::NoAtSign => "email address has no at sign",
            Error::NoHostPart => "email address has no host part",
            Error::NoUserPart => "email address has no user part",
            Error::NumericTld => "numeric TLD",
            Error::QuoteUnclosed => "email has an unclosed quotation mark",
            Error::TooManyLabels => "too many labels",
        };
        write!(f, "{}", error)
    }
}
