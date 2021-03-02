use core::convert::TryFrom;
use core::fmt;

use crate::parser::parse_domain;
use psl::{self, List, Psl};

pub use crate::{Error, Result};

/// Holds information about a particular DNS name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Name<'a> {
    domain: &'a str,
    root_offset: usize,
    suffix_offset: usize,
    suffix_is_known: bool,
}

impl<'a> TryFrom<&'a str> for Name<'a> {
    type Error = Error;

    fn try_from(domain: &'a str) -> Result<Self> {
        let root = List::new().domain(domain).ok_or(Error::InvalidDomain)?;
        let root_str = root.to_str();
        parse_domain(root_str)?;
        let root_offset = domain.len() - root_str.len();
        let suffix_offset = domain.len() - root.suffix().to_str().len();
        let suffix_is_known = root.suffix().is_known();
        Ok(Self {
            domain,
            root_offset,
            suffix_offset,
            suffix_is_known,
        })
    }
}

impl Name<'_> {
    pub fn as_str(&self) -> &str {
        &self.domain
    }

    pub fn root(&self) -> &str {
        &self.domain[self.root_offset..]
    }

    pub fn suffix(&self) -> &str {
        &self.domain[self.suffix_offset..]
    }

    pub fn suffix_is_known(&self) -> bool {
        self.suffix_is_known
    }
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
