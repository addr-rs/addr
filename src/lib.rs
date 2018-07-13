/*
extern crate psl;

use std::str::FromStr;
use std::fmt;

use psl::{Psl, List};

pub trait Domain<'a> {
    fn as_str(&self) -> &'a str;
    fn root(&self) -> psl::Domain<'a>;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct DomainName<'a> {
    str: &'a str,
    root: psl::Domain<'a>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct DnsName<'a> {
    str: &'a str,
    root: psl::Domain<'a>,
}

impl<'a> FromStr for DomainName<'a> {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match List::new().domain(str) {
            Some(root) => { Ok(DomainName { str, root }) }
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
    fn as_str(&self) -> &'a str {
        &self.str
    }

    fn root(&self) -> psl::Domain<'a> {
        self.root
    }
}

impl<'a> Domain<'a> for DnsName<'a> {
    fn as_str(&self) -> &'a str {
        &self.str
    }

    fn root(&self) -> psl::Domain<'a> {
        self.root
    }
}

impl<'a> fmt::Display for DomainName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}

impl<'a> fmt::Display for DnsName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}
*/
