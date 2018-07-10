#![no_std]

use core::str::FromStr;

pub trait Domain {
    fn is_icann(&self) -> bool;
    fn is_private(&self) -> bool;
    fn has_known_suffix(&self) -> bool;
}

pub struct DomainName;

pub struct DnsName;

impl FromStr for DomainName {
    type Err = ();

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        unimplemented!();
    }
}

impl FromStr for DnsName {
    type Err = ();

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        unimplemented!();
    }
}
