/*!
  Robust domain name parsing using the Public Suffix List

  This library allows you to easily and accurately parse any given domain name.

  ## Examples

  ```rust
  use addr::{parse_domain_name, parse_dns_name};

  # fn main() -> Result<(), Box<dyn std::error::Error>> {
  # #[cfg(feature = "psl")]
  # {
  // You can find out the root domain
  // or extension of any given domain name
  let domain = parse_domain_name("www.example.com")?;
  assert_eq!(domain.root(), Some("example.com"));
  assert_eq!(domain.suffix(), "com");

  let domain = parse_domain_name("www.食狮.中国")?;
  assert_eq!(domain.root(), Some("食狮.中国"));
  assert_eq!(domain.suffix(), "中国");

  let domain = parse_domain_name("www.xn--85x722f.xn--55qx5d.cn")?;
  assert_eq!(domain.root(), Some("xn--85x722f.xn--55qx5d.cn"));
  assert_eq!(domain.suffix(), "xn--55qx5d.cn");

  let domain = parse_domain_name("a.b.example.uk.com")?;
  assert_eq!(domain.root(), Some("example.uk.com"));
  assert_eq!(domain.suffix(), "uk.com");

  let name = parse_dns_name("_tcp.example.com.")?;
  assert_eq!(name.suffix(), Some("com."));

  // In any case if the domain's suffix is in the list
  // then this is definately a registrable domain name
  assert!(domain.has_known_suffix());
  # }
  # Ok(())
  # }
  ```
!*/

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

pub mod dns;
pub mod domain;
pub mod email;
pub mod error;
mod matcher;
#[cfg(feature = "net")]
pub mod net;
pub mod parser;
#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "psl")]
pub use crate::psl::{parse_dns_name, parse_domain_name, parse_email_address};

/// The static implementation of the public suffix list
#[cfg(feature = "psl")]
pub mod psl {
    use crate::parser::{DnsName, DomainName, EmailAddress};
    use crate::{dns, domain, email, Result};

    pub use psl::List;

    pub fn parse_domain_name(input: &str) -> Result<'_, domain::Name<'_>> {
        List.parse_domain_name(input)
    }

    pub fn parse_dns_name(input: &str) -> Result<'_, dns::Name<'_>> {
        List.parse_dns_name(input)
    }

    pub fn parse_email_address(input: &str) -> Result<'_, email::Address<'_>> {
        List.parse_email_address(input)
    }
}

/// The dynamic implementation of the public suffix list
#[cfg(feature = "publicsuffix")]
pub mod publicsuffix {
    pub use publicsuffix::{IcannList, List, PrivateList};
}

/// Custom result type
pub type Result<'a, T> = core::result::Result<T, error::Error<'a>>;
