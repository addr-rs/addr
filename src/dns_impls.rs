use std::cmp;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use errors::ErrorKind;
use parser::{parse_domain, to_targetcase};
use psl::{self, List, Psl};
use DnsName;

pub use errors::{Error, Result};

impl FromStr for DnsName {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        use inner::Dns;
        let full = to_targetcase(input);
        let inner = Dns::try_new_or_drop(full, |full| match List::new().domain(&full) {
            Some(root) => {
                if parse_domain(root.to_str()).is_err() {
                    return Err(Error::from(ErrorKind::InvalidDomain(input.into())));
                }
                Ok(root)
            }
            None => Err(Error::from(ErrorKind::InvalidDomain(input.into()))),
        })?;
        Ok(DnsName { inner })
    }
}

impl DnsName {
    pub fn as_str(&self) -> &str {
        self.inner.head()
    }

    pub fn root(&self) -> psl::Domain<'_> {
        let rental = unsafe { self.inner.all_erased() };
        *rental.root
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
