#![no_std]
#![cfg_attr(feature = "dynamic", crate_type = "dylib")]

#[cfg(feature = "list")]
extern crate serde;
#[cfg(feature = "list")]
#[macro_use]
extern crate psl_codegen;

#[cfg(feature = "list")]
mod list;

use core::{str, fmt};

#[cfg(feature = "list")]
pub use list::List;

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
    str: &'a str,
    typ: Option<Type>,
}

/// A registrable domain name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Domain<'a> {
    str: &'a str,
    suf: Suffix<'a>,
}

/// A list of all public suffices
pub trait Psl {
    /// Finds the suffix of the given input labels
    ///
    /// # Assumptions
    ///
    /// *NB:* `domain` must be in lowercase, in unicode and with no trailing dot
    fn find(&self, domain: &str) -> Info;

    /// Get the public suffix of the domain
    /// 
    /// *NB:* `domain` must be in lowercase, in unicode and with no trailing dot
    fn suffix<'a>(&self, domain: &'a str) -> Option<Suffix<'a>> {
        let Info { len, typ } = self.find(domain);
        if len == 0 {
            return None;
        }
        let offset = domain.len() - len;
        let bytes = domain.as_bytes();
        let str = str::from_utf8(&bytes[offset..]).ok()?;
        Some(Suffix { str, typ })
    }

    /// Get the registrable domain
    /// 
    /// *NB:* `domain` must be in lowercase, in unicode and with no trailing dot
    fn domain<'a>(&self, domain: &'a str) -> Option<Domain<'a>> {
        let suf = self.suffix(domain)?;
        let mut labels = domain
            .trim_right_matches(suf.as_str())
            .split('.')
            .rev();
        // remove trailing dot
        labels.next()?;
        let offset = domain.len() - (suf.as_str().len() + labels.next()?.len() + 1);
        let bytes = domain.as_bytes();
        let str = str::from_utf8(&bytes[offset..]).ok()?;
        Some(Domain { str, suf })
    }
}

impl<'a> Suffix<'a> {
    pub fn as_str(&self) -> &str {
        &self.str
    }

    pub fn typ(&self) -> Option<Type> {
        self.typ
    }

    pub fn is_known(&self) -> bool {
        self.typ.is_some()
    }
}

impl<'a> fmt::Display for Suffix<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}

impl<'a> Domain<'a> {
    pub fn as_str(&self) -> &str {
        &self.str
    }

    pub fn suffix(&self) -> Suffix<'a> {
        self.suf
    }
}

impl<'a> fmt::Display for Domain<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::Icann
    }
}
