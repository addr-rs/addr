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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Type {
    Icann,
    Private,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Info {
    Suffix(usize, Type),
    Incomplete,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Suffix<'a> {
    str: &'a str,
    typ: Option<Type>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
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
    /// - The input is an `Iterator` of domain labels.
    /// - The labels are in reverse order. That is, `&["com", "example"]` instead of
    /// `&["example", "com"].
    /// - The labels are in lowercase.
    /// - The labels are in unicode, rather than punnycode.
    fn find_unchecked<'a, T>(&self, labels: T) -> Option<Info>
        where T: IntoIterator<Item = &'a str>;

    /// Get the public suffix of the domain
    /// 
    /// *NB:* `domain` must be in lowercase
    fn public_suffix<'a>(&self, domain: &'a str) -> Option<Suffix<'a>> {
        if domain.starts_with('.') || domain.contains("..") {
            return None;
        }
        let mut labels = domain
            .trim_right_matches('.')
            .split('.')
            .rev()
            .peekable();
        if labels.peek().is_none() { return None; }
        let (len, typ) = match self.find_unchecked(labels.clone()) {
            Some(info) => {
                match info {
                    Info::Suffix(len, typ) => { (len, Some(typ)) }
                    Info::Incomplete => { return None; }
                }
            }
            None => { (1, None) }
        };
        let mut slen = 0;
        if domain.ends_with('.') {
            slen += 1;
        };
        for label in labels.take(len) {
            slen += label.len() + 1;
        }
        let offset = domain.len() + 1 - slen;
        let bytes = domain.as_bytes();
        let str = str::from_utf8(&bytes[offset..]).ok()?;
        Some(Suffix { str, typ })
    }

    /// Get the registrable domain
    /// 
    /// *NB:* `domain` must be in lowercase
    fn registrable_domain<'a>(&self, domain: &'a str) -> Option<Domain<'a>> {
        let suf = self.public_suffix(domain)?;
        let label = domain
            .trim_right_matches(suf.as_str())
            .trim_right_matches('.')
            .split('.')
            .last()?;
        if label.is_empty() { return None; }
        let offset = domain.len() - (suf.as_str().len() + label.len() + 1);
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
        Suffix { ..self.suf }
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

impl Default for Info {
    fn default() -> Self {
        Info::Incomplete
    }
}
