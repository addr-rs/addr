//! DNS types

use crate::error::Result;
use crate::matcher;
use core::{fmt, str};
use psl_types::{List, Suffix, Type};

/// Holds information about a particular DNS name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Name<'a> {
    full: &'a str,
    suffix: Option<Suffix<'a>>,
}

impl<'a> Name<'a> {
    pub(crate) fn parse<T: List + ?Sized>(list: &T, name: &'a str) -> Result<Name<'a>> {
        matcher::is_dns_name(name)?;
        Ok(Self {
            full: name,
            suffix: list.suffix(name.as_bytes()),
        })
    }

    /// Full dns name as a `str`
    pub const fn as_str(&self) -> &str {
        &self.full
    }

    /// The root domain (the registrable part)
    pub fn root(&self) -> Option<&str> {
        let offset = self.prefix()?.rfind('.').map(|x| x + 1).unwrap_or_default();
        self.full.get(offset..)
    }

    fn prefix(&self) -> Option<&str> {
        let prefix = self
            .full
            .trim_end_matches(self.suffix()?)
            .trim_end_matches('.');
        if prefix.is_empty() {
            return None;
        }
        Some(prefix)
    }

    /// The domain name suffix (extension)
    pub fn suffix(&self) -> Option<&str> {
        let bytes = self.suffix.as_ref()?.as_bytes();
        str::from_utf8(bytes).ok()
    }

    /// Whether the suffix of the domain name is in the Public Suffix List
    pub fn has_known_suffix(&self) -> bool {
        if let Some(suffix) = self.suffix {
            suffix.is_known()
        } else {
            false
        }
    }

    /// Whether this an ICANN delegated suffix
    ///
    /// ICANN domains are those delegated by ICANN or part of the IANA root
    /// zone database
    pub fn is_icann(&self) -> bool {
        if let Some(suffix) = self.suffix {
            suffix.typ() == Some(Type::Icann)
        } else {
            false
        }
    }

    /// Whether this is a private party delegated suffix
    ///
    /// PRIVATE domains are amendments submitted by the domain holder, as an
    /// expression of how they operate their domain security policy
    pub fn is_private(&self) -> bool {
        if let Some(suffix) = self.suffix {
            suffix.typ() == Some(Type::Private)
        } else {
            false
        }
    }
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.full)
    }
}

impl PartialEq<&str> for Name<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.full == *other
    }
}
