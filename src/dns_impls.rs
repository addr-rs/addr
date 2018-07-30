use std::fmt;
use std::str::FromStr;
use std::cmp::PartialEq;

use psl::{self, Psl, List};
use errors::ErrorKind;
use parser::{parse_domain, to_targetcase};
use DnsName;

pub use errors::{Result, Error};

impl FromStr for DnsName {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        use inner::Dns;
        let full = to_targetcase(input);
        let inner = Dns::try_new_or_drop(full, |full| {
            match List::new().domain(&full) {
                Some(root) => {
                    if parse_domain(root.to_str()).is_err() {
                        return Err(Error::from(ErrorKind::InvalidDomain(input.into())));
                    }
                    Ok(root)
                }
                None => { Err(Error::from(ErrorKind::InvalidDomain(input.into()))) }
            }
        })?;
        Ok(DnsName { inner })
    }
}

impl DnsName {
    pub fn as_str(&self) -> &str {
        self.inner.head()
    }

    pub fn root<'a>(&'a self) -> psl::Domain<'a> {
        let rental = unsafe { self.inner.all_erased() };
        *rental.root
    }
}

impl fmt::Display for DnsName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq for DnsName {
    fn eq(&self, other: &DnsName) -> bool {
        self.as_str() == other.as_str()
    }
}
