/*!
  Robust domain name parsing using the Public Suffix List

  This library allows you to easily and accurately parse any given domain name.

  ## Examples

  ```rust
  extern crate addr;

  use addr::{DomainName, DnsName, Email};
  # use addr::Result;

  # fn main() {
    // You can find out the root domain
    // or extension of any given domain name
    let domain: DomainName = "www.example.com".parse().unwrap();
    assert_eq!(domain.root(), "example.com");
    assert_eq!(domain.root().suffix(), "com");

    let domain: DomainName = "www.食狮.中国".parse().unwrap();
    assert_eq!(domain.root(), "xn--85x722f.xn--fiqs8s");
    assert_eq!(domain.root().suffix(), "xn--fiqs8s");

    let domain: DomainName = "www.xn--85x722f.xn--55qx5d.cn".parse().unwrap();
    assert_eq!(domain.root(), "xn--85x722f.xn--55qx5d.cn");
    assert_eq!(domain.root().suffix(), "xn--55qx5d.cn");

    let domain: DomainName = "a.b.example.uk.com".parse().unwrap();
    assert_eq!(domain.root(), "example.uk.com");
    assert_eq!(domain.root().suffix(), "uk.com");

    let name: DnsName = "_tcp.example.com.".parse().unwrap();
    assert_eq!(name.root(), "example.com.");
    assert_eq!(name.root().suffix(), "com.");

    let email: Email = "чебурашка@ящик-с-апельсинами.рф".parse().unwrap();
    assert_eq!(email.user(), "чебурашка");
    assert_eq!(email.host(), "xn-----8kcayoeblonkwzf2jqc1b.xn--p1ai");

    // In any case if the domain's suffix is in the list
    // then this is definately a registrable domain name
    assert!(domain.root().suffix().is_known());
  # }
  ```
!*/

#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate psl;
#[macro_use]
extern crate rental;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate idna;

mod parser;
mod host;
mod domain_impls;
mod dns_impls;
mod email;
pub mod errors;

use std::net::IpAddr;

pub use errors::{Result, Error};

rental! {
    mod inner {
        use psl::Domain as Root;

        #[rental(debug)]
        pub struct Domain {
            full: String,
            root: Root<'full>,
        }

        #[rental(debug)]
        pub struct Dns {
            full: String,
            root: Root<'full>,
        }
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
