use std::fmt;
use std::str::FromStr;
use std::cmp;
use std::hash::{Hash, Hasher};

use psl::{self, Psl, List};
use errors::{Error, Result, ErrorKind};
use parser::parse_domain;
use DomainName;

impl FromStr for DomainName {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        use inner::Domain;
        match parse_domain(input) {
            Ok(domain) => {
                let inner = Domain::try_new_or_drop(domain, |full| {
                    match List.domain(&full) {
                        Some(root) => { Ok(root) }
                        None => { Err(Error::from(ErrorKind::InvalidDomain(input.into()))) }
                    }
                })?;
                Ok(DomainName { inner })
            }
            Err(_) => {
                Err(ErrorKind::InvalidDomain(input.into()).into())
            }
        }
    }
}

impl DomainName {
    pub fn as_str(&self) -> &str {
        self.inner.head()
    }

    pub fn root<'a>(&'a self) -> psl::Domain<'a> {
        let rental = unsafe { self.inner.all_erased() };
        *rental.root
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

impl cmp::Eq for DomainName { }

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
