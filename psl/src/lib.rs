#![no_std]
#![forbid(unsafe_code)]

mod list;
#[cfg(feature = "serde")]
mod serde;

use core::cmp::PartialEq;
use core::{fmt, str};

pub use list::List;

/// A list of all public suffices
pub trait Psl {
    /// Finds the suffix of the given input labels
    ///
    /// # Assumptions
    ///
    /// *NB:* `domain` must be a valid domain name in lowercase
    fn find(&self, domain: &[u8]) -> Info;

    /// Get the public suffix of the domain
    ///
    /// *NB:* `domain` must be a valid domain name in lowercase
    #[inline]
    fn suffix<'a>(&self, domain: &'a str) -> Option<Suffix<'a>> {
        let domain = domain.as_bytes();
        let Info { len, typ } = self.find(domain);
        if len == 0 {
            return None;
        }
        let offset = domain.len() - len;
        let bytes = &domain[offset..];
        Some(Suffix { bytes, typ })
    }

    /// Get the registrable domain
    ///
    /// *NB:* `domain` must be a valid domain name in lowercase
    #[inline]
    fn domain<'a>(&self, domain: &'a str) -> Option<Domain<'a>> {
        let suffix = self.suffix(domain)?;
        let domain = domain.as_bytes();
        let domain_len = domain.len();
        let suffix_len = suffix.bytes.len();
        if domain_len < suffix_len + 2 {
            return None;
        }
        let offset = domain_len - (1 + suffix_len);
        let subdomain = &domain[..offset];
        let root_label = subdomain.rsplitn(2, |x| *x == b'.').next()?;
        let registrable_len = root_label.len() + 1 + suffix_len;
        let offset = domain_len - registrable_len;
        let bytes = &domain[offset..];
        Some(Domain { bytes, suffix })
    }
}

/// Type of suffix
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Type {
    Icann,
    Private,
}

/// Information about the suffix
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Info {
    pub len: usize,
    pub typ: Option<Type>,
}

/// The suffix of a domain name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Suffix<'a> {
    bytes: &'a [u8],
    typ: Option<Type>,
}

impl<'a> Suffix<'a> {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn to_str(&self) -> &str {
        str::from_utf8(&self.bytes).unwrap()
    }

    #[inline]
    pub fn typ(&self) -> Option<Type> {
        self.typ
    }

    #[inline]
    pub fn is_known(&self) -> bool {
        self.typ.is_some()
    }
}

impl<'a> fmt::Display for Suffix<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl<'a, 'b> PartialEq<&'a str> for Suffix<'b> {
    fn eq(&self, other: &&'a str) -> bool {
        self.to_str() == *other
    }
}

/// A registrable domain name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Domain<'a> {
    bytes: &'a [u8],
    suffix: Suffix<'a>,
}

impl<'a> Domain<'a> {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn to_str(&self) -> &str {
        str::from_utf8(&self.bytes).unwrap()
    }

    #[inline]
    pub fn suffix(&self) -> Suffix<'a> {
        self.suffix
    }
}

impl<'a> fmt::Display for Domain<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl<'a, 'b> PartialEq<&'a str> for Domain<'b> {
    fn eq(&self, other: &&'a str) -> bool {
        self.to_str() == *other
    }
}
