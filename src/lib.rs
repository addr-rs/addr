/*!
  Robust domain name parsing using the Public Suffix List

  This library allows you to easily and accurately parse any given domain name.

  ## Examples

  ```rust
  # fn main() -> addr::Result<()> {
  use addr::{dns, domain, Error};

  // You can find out the root domain
  // or extension of any given domain name
  let domain = domain::Name::parse("www.example.com")?;
  assert_eq!(domain.root(), "example.com");
  assert_eq!(domain.suffix(), "com");

  let name = idna::domain_to_ascii("www.食狮.中国")
    .map_err(|_| Error::InvalidDomain)?;
  let domain = domain::Name::parse(name.as_str())?;
  assert_eq!(domain.root(), "xn--85x722f.xn--fiqs8s");
  assert_eq!(domain.suffix(), "xn--fiqs8s");

  let domain = domain::Name::parse("www.xn--85x722f.xn--55qx5d.cn")?;
  assert_eq!(domain.root(), "xn--85x722f.xn--55qx5d.cn");
  assert_eq!(domain.suffix(), "xn--55qx5d.cn");

  let domain = domain::Name::parse("a.b.example.uk.com")?;
  assert_eq!(domain.root(), "example.uk.com");
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
mod parser;
#[cfg(feature = "serde")]
mod serde;

use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[non_exhaustive]
pub enum Error {
    DomainNotAscii,
    NameTooLong,
    EmptyLabel,
    EmptyName,
    IllegalCharacter,
    InvalidDomain,
    LabelEndNotAlnum,
    LabelStartNotAlnum,
    LabelTooLong,
    NumericTld,
    TooManyLabels,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error = match self {
            Error::DomainNotAscii => "domain not ascii",
            Error::NameTooLong => "name too long",
            Error::EmptyLabel => "domain contains empty label",
            Error::EmptyName => "name is empty",
            Error::IllegalCharacter => "domain contains illegal characters",
            Error::InvalidDomain => "invalid domain name",
            Error::LabelEndNotAlnum => "label does not start with an alphanumeric character",
            Error::LabelStartNotAlnum => "label does not end with a alphanumeric character",
            Error::LabelTooLong => "label too long",
            Error::NumericTld => "numeric TLD",
            Error::TooManyLabels => "too many labels",
        };
        write!(f, "{}", error)
    }
}
