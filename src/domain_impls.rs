use std::cmp;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crate::parser::parse_domain;
use crate::DomainName;
use crate::{Error, Result};
use psl::{self, List, Psl};

impl FromStr for DomainName {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        use crate::inner::Domain;
        match parse_domain(input) {
            Ok(full) => {
                let root = match List.domain(&full) {
                    Some(root) => root,
                    None => {
                        return Err(Error::InvalidDomain(input.into()));
                    }
                };
                let root_offset = full.len() - root.to_str().len();
                let suffix_offset = full.len() - root.suffix().to_str().len();
                let suffix_is_known = root.suffix().is_known();
                let inner = Domain {
                    full,
                    root_offset,
                    suffix_offset,
                    suffix_is_known,
                };
                Ok(DomainName { inner })
            }
            Err(_) => Err(Error::InvalidDomain(input.into())),
        }
    }
}

impl DomainName {
    pub fn as_str(&self) -> &str {
        &self.inner.full
    }

    pub fn root(&self) -> &str {
        &self.inner.full[self.inner.root_offset..]
    }

    pub fn suffix(&self) -> &str {
        &self.inner.full[self.inner.suffix_offset..]
    }

    pub fn suffix_is_known(&self) -> bool {
        self.inner.suffix_is_known
    }
}

impl fmt::Display for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl cmp::PartialEq for DomainName {
    fn eq(&self, other: &DomainName) -> bool {
        self.as_str() == other.as_str()
    }
}

impl cmp::Eq for DomainName {}

impl cmp::PartialOrd for DomainName {
    fn partial_cmp(&self, other: &DomainName) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for DomainName {
    fn cmp(&self, other: &DomainName) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for DomainName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}
