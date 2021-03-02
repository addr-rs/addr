/*!
  Robust domain name parsing using the Public Suffix List

  This library allows you to easily and accurately parse any given domain name.

  ## Examples

  ```rust
  use addr::{dns, domain};
  use core::convert::TryFrom;

  // You can find out the root domain
  // or extension of any given domain name
  let domain = domain::Name::try_from("www.example.com").unwrap();
  assert_eq!(domain.root(), "example.com");
  assert_eq!(domain.suffix(), "com");

  let punycode = idna::domain_to_ascii("www.食狮.中国").unwrap();
  let domain = domain::Name::try_from(punycode.as_str()).unwrap();
  assert_eq!(domain.root(), "xn--85x722f.xn--fiqs8s");
  assert_eq!(domain.suffix(), "xn--fiqs8s");

  let domain = domain::Name::try_from("www.xn--85x722f.xn--55qx5d.cn").unwrap();
  assert_eq!(domain.root(), "xn--85x722f.xn--55qx5d.cn");
  assert_eq!(domain.suffix(), "xn--55qx5d.cn");

  let domain = domain::Name::try_from("a.b.example.uk.com").unwrap();
  assert_eq!(domain.root(), "example.uk.com");
  assert_eq!(domain.suffix(), "uk.com");

  let name = dns::Name::try_from("_tcp.example.com.").unwrap();
  assert_eq!(name.root(), "example.com.");
  assert_eq!(name.suffix(), "com.");

  // In any case if the domain's suffix is in the list
  // then this is definately a registrable domain name
  assert!(domain.suffix_is_known());
  ```
!*/

#![no_std]
#![forbid(unsafe_code)]

pub mod dns;
pub mod domain;
mod parser;

use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    NotAscii,
    InvalidDomain,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error = match self {
            Error::NotAscii => "domain is not in ascii",
            Error::InvalidDomain => "invalid domain name",
        };
        write!(f, "{}", error)
    }
}
