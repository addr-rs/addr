/*!
  Robust domain name parsing using the Public Suffix List

  This library allows you to easily and accurately parse any given domain name.

  ## Examples

  ```rust
  extern crate addr;

  use addr::{DomainName, DnsName, Email};
  # use addr::Result;

  # fn main() -> Result<()> {
    // You can find out the root domain
    // or extension of any given domain name
    let domain: DomainName = "www.example.com".parse().unwrap();
    assert_eq!(domain.root(), "example.com");
    assert_eq!(domain.root().suffix(), "com");

    let domain: DomainName = "www.食狮.中国".parse()?;
    assert_eq!(domain.root(), "xn--85x722f.xn--fiqs8s");
    assert_eq!(domain.root().suffix(), "xn--fiqs8s");

    let domain: DomainName = "www.xn--85x722f.xn--55qx5d.cn".parse()?;
    assert_eq!(domain.root(), "xn--85x722f.xn--55qx5d.cn");
    assert_eq!(domain.root().suffix(), "xn--55qx5d.cn");

    let domain: DomainName = "a.b.example.uk.com".parse()?;
    assert_eq!(domain.root(), "example.uk.com");
    assert_eq!(domain.root().suffix(), "uk.com");

    let name: DnsName = "_tcp.example.com.".parse()?;
    assert_eq!(name.root(), "example.com.");
    assert_eq!(name.root().suffix(), "com.");

    let email: Email = "чебурашка@ящик-с-апельсинами.рф".parse()?;
    assert_eq!(email.user(), "чебурашка");
    assert_eq!(email.host(), "xn-----8kcayoeblonkwzf2jqc1b.xn--p1ai");

    // In any case if the domain's suffix is in the list
    // then this is definately a registrable domain name
    assert!(domain.root().suffix().is_known());
  # Ok(())
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
pub mod errors;

use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;
use std::cmp::PartialEq;

use psl::{Psl, List};
use regex::RegexSet;
use errors::ErrorKind;
use parser::{parse_domain, to_targetcase};

pub use errors::{Result, Error};

lazy_static! {
    // Regex for matching the local-part of an
    // email address
    static ref LOCAL: RegexSet = {
        // these characters can be anywhere in the expresion
        let global = r#"[[:alnum:]!#$%&'*+/=?^_`{|}~-]"#;
        // non-ascii characters (an also be unquoted)
        let non_ascii = r#"[^\x00-\x7F]"#;
        // the pattern to match
        let quoted = r#"["(),\\:;<>@\[\]. ]"#;
        // combined regex
        let combined = format!(r#"({}*{}*)"#, global, non_ascii);

        let exprs = vec![
            // can be any combination of allowed characters
            format!(r#"^{}+$"#, combined),
            // can be any combination of allowed charaters
            // separated by a . in between
            format!(r#"^({0}+[.]?{0}+)+$"#, combined),
            // can be a quoted string with allowed plus
            // additional characters
            format!(r#"^"({}*{}*)*"$"#, combined, quoted),
        ];

        RegexSet::new(exprs).unwrap()
    };
}

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
///
/// This is created by `List::parse_dns_name`.
//#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[derive(Debug)]
pub struct DnsName {
    inner: inner::Dns,
}

/// Holds information about a particular host
///
/// This is created by `List::parse_host`.
#[derive(Debug)]
pub enum Host {
    Ip(IpAddr),
    Domain(DomainName),
}

#[derive(Debug)]
pub struct Email {
    name: String,
    host: Host,
}

impl FromStr for DomainName {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        use inner::Domain;
        match parse_domain(input) {
            Ok(domain) => {
                let inner = Domain::try_new_or_drop(domain, |full| {
                    match List.domain(&full) {
                        Some(root) => { Ok(root) }
                        None => { Err(Error::from(ErrorKind::InvalidDomain(input.into()))) }
                    }
                })?;
                Ok(DomainName { inner })
            }
            Err(_) => {
                Err(ErrorKind::InvalidDomain(input.into()).into())
            }
        }
    }
}

impl FromStr for DnsName {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        use inner::Dns;
        let full = to_targetcase(input);
        let inner = Dns::try_new_or_drop(full, |full| {
            match List::new().domain(&full) {
                Some(root) => {
                    if parse_domain(root.to_str()).is_err() {
                        return Err(Error::from(ErrorKind::InvalidDomain(input.into())));
                    }
                    Ok(root)
                }
                None => { Err(Error::from(ErrorKind::InvalidDomain(input.into()))) }
            }
        })?;
        Ok(DnsName { inner })
    }
}

impl DnsName {
    pub fn as_str(&self) -> &str {
        self.inner.head()
    }

    pub fn root<'a>(&'a self) -> psl::Domain<'a> {
        let rental = unsafe { self.inner.all_erased() };
        *rental.root
    }
}

impl FromStr for Host {
    type Err = Error;

    fn from_str(mut host: &str) -> Result<Host> {
        if let Ok(domain) = DomainName::from_str(host) {
            return Ok(Host::Domain(domain));
        }
        if host.starts_with("[") 
            && !host.starts_with("[[")
                && host.ends_with("]")
                && !host.ends_with("]]")
                {
                    host = host
                        .trim_left_matches("[")
                        .trim_right_matches("]");
                };
        if let Ok(ip) = IpAddr::from_str(host) {
            return Ok(Host::Ip(ip));
        }
        Err(ErrorKind::InvalidHost.into())
    }
}

impl Host {
    /// A convenient method to simply check if a host is an IP address
    pub fn is_ip(&self) -> bool {
        if let &Host::Ip(_) = self {
            return true;
        }
        false
    }

    /// A convenient method to simply check if a host is a domain name
    pub fn is_domain(&self) -> bool {
        if let &Host::Domain(_) = self {
            return true;
        }
        false
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Host::Ip(ref ip) => write!(f, "{}", ip),
            &Host::Domain(ref domain) => write!(f, "{}", domain),
        }
    }
}

impl DomainName {
    pub fn as_str(&self) -> &str {
        self.inner.head()
    }

    pub fn root<'a>(&'a self) -> psl::Domain<'a> {
        let rental = unsafe { self.inner.all_erased() };
        *rental.root
    }
}

impl FromStr for Email {
    type Err = Error;

    /// Extracts Host from an email address
    ///
    /// This method can also be used, simply to validate an email address.
    /// If it returns an error, the email address is not valid.
    // https://en.wikipedia.org/wiki/Email_address#Syntax
    // https://en.wikipedia.org/wiki/International_email#Email_addresses
    // http://girders.org/blog/2013/01/31/dont-rfc-validate-email-addresses/
    // https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
    // https://hackernoon.com/the-100-correct-way-to-validate-email-addresses-7c4818f24643#.pgcir4z3e
    // http://haacked.com/archive/2007/08/21/i-knew-how-to-validate-an-email-address-until-i.aspx/
    // https://tools.ietf.org/html/rfc6530#section-10.1
    // http://rumkin.com/software/email/rules.php
    fn from_str(address: &str) -> Result<Email> {
        let mut parts = address.rsplitn(2, "@");
        let host = match parts.next() {
            Some(host) => host,
            None => { return Err(ErrorKind::InvalidEmail.into()); }
        };
        let local = match parts.next() {
            Some(local) => local,
            None => { return Err(ErrorKind::InvalidEmail.into()); }
        };
        if local.chars().count() > 64
            || address.chars().count() > 254
            || (!local.starts_with('"') && local.contains(".."))
            || !LOCAL.is_match(local)
        {
            return Err(ErrorKind::InvalidEmail.into());
        }
        let host = Host::from_str(host)?;
        let name = local.to_owned();
        Ok(Email { name, host })
    }
}

impl Email {
    pub fn user(&self) -> &str {
        &self.name
    }

    pub fn host(&self) -> &Host {
        &self.host
    }
}

impl fmt::Display for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for DnsName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.host)
    }
}

impl PartialEq for DomainName {
    fn eq(&self, other: &DomainName) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq for DnsName {
    fn eq(&self, other: &DnsName) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for Host {
    fn eq(&self, other: &str) -> bool {
        self.to_string() == other
    }
}
