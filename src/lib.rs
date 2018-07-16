extern crate psl;
#[macro_use]
extern crate rental;

use std::fmt;

use psl::{Psl, List};

pub use name::DomainName;

pub trait Domain<'a> {
    fn as_str(&self) -> &str;
    fn root(&self) -> psl::Domain<'a>;
}

rental! {
    mod name {
        use psl::Domain;

        #[rental]
        pub struct DomainName {
            full: String,
            root: psl::Domain<'full>,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct DnsName<'a> {
    str: &'a str,
    root: psl::Domain<'a>,
}

/*
impl DomainName {
    fn from_str(str: &str) -> Result<Self, ()> {
        let full = str.to_lowercase();
        match List::new().domain(&full) {
            Some(root) => { Ok(DomainName { full, root }) }
            None => { Err(()) }
        }
    }
}

impl<'a> FromStr for DnsName<'a> {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match List::new().domain(str) {
            Some(root) => { Ok(DnsName { str, root }) }
            None => { Err(()) }
        }
    }
}

impl<'a> Domain<'a> for DomainName<'a> {
    fn as_str(&self) -> &str {
        &self.full
    }

    fn root(&self) -> psl::Domain<'a> {
        self.root
    }
}

impl<'a> Domain<'a> for DnsName<'a> {
    fn as_str(&self) -> &str {
        &self.str
    }

    fn root(&self) -> psl::Domain<'a> {
        self.root
    }
}

impl<'a> fmt::Display for DomainName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.full)
    }
}

impl<'a> fmt::Display for DnsName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}
*/
