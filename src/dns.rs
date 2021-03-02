use crate::parser::parse_domain;
use crate::{Error, Result};
use core::convert::TryFrom;
use core::fmt;

/// Holds information about a particular DNS name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Name<'a> {
    full: &'a str,
    root: psl::Domain<'a>,
}

impl<'a> TryFrom<&'a str> for Name<'a> {
    type Error = Error;

    fn try_from(punycode: &'a str) -> Result<Self> {
        let name = Self {
            root: psl::domain(punycode.as_bytes()).ok_or(Error::InvalidDomain)?,
            full: punycode,
        };
        parse_domain(name.root())?;
        Ok(name)
    }
}

impl Name<'_> {
    pub fn as_str(&self) -> &str {
        &self.full
    }

    pub fn root(&self) -> &str {
        let offset = self.full.len() - self.root.as_bytes().len();
        &self.full[offset..]
    }

    pub fn suffix(&self) -> &str {
        let offset = self.full.len() - self.root.suffix().as_bytes().len();
        &self.full[offset..]
    }

    pub fn suffix_is_known(&self) -> bool {
        self.root.suffix().is_known()
    }

    pub fn is_icann(&self) -> bool {
        self.root
            .suffix()
            .typ()
            .filter(|t| *t == psl::Type::Icann)
            .is_some()
    }

    pub fn is_private(&self) -> bool {
        self.root
            .suffix()
            .typ()
            .filter(|t| *t == psl::Type::Private)
            .is_some()
    }
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.full)
    }
}
