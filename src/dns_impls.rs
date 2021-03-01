use std::cmp;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crate::parser::{parse_domain, to_targetcase};
use crate::DnsName;
use psl::{self, List, Psl};

pub use crate::{Error, Result};

impl FromStr for DnsName {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        use crate::inner::Dns;
        let full = to_targetcase(input);
        let (root_offset, suffix_offset, suffix_is_known) = match List::new().domain(&full) {
            Some(root) => {
                let root_str = root.to_str();
                if parse_domain(root_str).is_err() {
                    return Err(Error::InvalidDomain(input.into()));
                }
                let root_offset = full.len() - root_str.len();
                let suffix_offset = full.len() - root.suffix().to_str().len();
                (root_offset, suffix_offset, root.suffix().is_known())
            }
            None => {
                return Err(Error::InvalidDomain(input.into()));
            }
        };
        let inner = Dns {
            full,
            root_offset,
            suffix_offset,
            suffix_is_known,
        };
        Ok(DnsName { inner })
    }
}

impl DnsName {
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

impl fmt::Display for DnsName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl cmp::PartialEq for DnsName {
    fn eq(&self, other: &DnsName) -> bool {
        self.as_str() == other.as_str()
    }
}

impl cmp::Eq for DnsName {}

impl cmp::PartialOrd for DnsName {
    fn partial_cmp(&self, other: &DnsName) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for DnsName {
    fn cmp(&self, other: &DnsName) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for DnsName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}
