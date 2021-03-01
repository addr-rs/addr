/*!
  Robust domain name parsing using the Public Suffix List

  This library allows you to easily and accurately parse any given domain name.

  ## Examples

  ```rust
  use addr::{DomainName, DnsName, Email};
  # use addr::Result;

  # fn main() {
    // You can find out the root domain
    // or extension of any given domain name
    let domain: DomainName = "www.example.com".parse().unwrap();
    assert_eq!(domain.root(), "example.com");
    assert_eq!(domain.suffix(), "com");

    let domain: DomainName = "www.食狮.中国".parse().unwrap();
    assert_eq!(domain.root(), "xn--85x722f.xn--fiqs8s");
    assert_eq!(domain.suffix(), "xn--fiqs8s");

    let domain: DomainName = "www.xn--85x722f.xn--55qx5d.cn".parse().unwrap();
    assert_eq!(domain.root(), "xn--85x722f.xn--55qx5d.cn");
    assert_eq!(domain.suffix(), "xn--55qx5d.cn");

    let domain: DomainName = "a.b.example.uk.com".parse().unwrap();
    assert_eq!(domain.root(), "example.uk.com");
    assert_eq!(domain.suffix(), "uk.com");

    let name: DnsName = "_tcp.example.com.".parse().unwrap();
    assert_eq!(name.root(), "example.com.");
    assert_eq!(name.suffix(), "com.");

    let email: Email = "чебурашка@ящик-с-апельсинами.рф".parse().unwrap();
    assert_eq!(email.user(), "чебурашка");
    assert_eq!(email.host(), "xn-----8kcayoeblonkwzf2jqc1b.xn--p1ai");

    // In any case if the domain's suffix is in the list
    // then this is definately a registrable domain name
    assert!(domain.suffix_is_known());
  # }
  ```
!*/

#![recursion_limit = "1024"]

mod dns_impls;
mod domain_impls;
mod email;
pub mod errors;
mod host;
mod parser;

use std::net::IpAddr;

pub use errors::Error;

pub type Result<T> = std::result::Result<T, Error>;

mod inner {
    #[derive(Debug)]
    pub struct Domain {
        pub full: String,
        pub root_offset: usize,
        pub suffix_offset: usize,
        pub suffix_is_known: bool,
    }

    #[derive(Debug)]
    pub struct Dns {
        pub full: String,
        pub root_offset: usize,
        pub suffix_offset: usize,
        pub suffix_is_known: bool,
    }
}

#[derive(Debug)]
pub struct DomainName {
    inner: inner::Domain,
}

/// Holds information about a particular DNS name
#[derive(Debug)]
pub struct DnsName {
    inner: inner::Dns,
}

/// Holds information about a particular host
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Host {
    Ip(IpAddr),
    Domain(DomainName),
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Email {
    name: String,
    host: Host,
}
