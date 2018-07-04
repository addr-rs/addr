#![no_std]

use core::str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Icann,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Info {
    Suffix(usize, Type),
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Suffix<'a> {
    str: &'a str,
    typ: Option<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    fn find_unchecked<'a>(&self, labels: impl Iterator<Item=&'a str>) -> Option<Info>;

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

impl<'a> Domain<'a> {
    pub fn as_str(&self) -> &str {
        &self.str
    }

    pub fn suffix(&self) -> Suffix<'a> {
        Suffix { ..self.suf }
    }
}
