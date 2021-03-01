use std::cmp::PartialEq;
use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

use crate::{DomainName, Host};
use crate::{Error, Result};

impl FromStr for Host {
    type Err = Error;

    fn from_str(mut host: &str) -> Result<Host> {
        if let Ok(domain) = DomainName::from_str(host) {
            return Ok(Host::Domain(domain));
        }
        if host.starts_with('[')
            && !host.starts_with("[[")
            && host.ends_with(']')
            && !host.ends_with("]]")
        {
            host = host.trim_start_matches('[').trim_end_matches(']');
        };
        if let Ok(ip) = IpAddr::from_str(host) {
            return Ok(Host::Ip(ip));
        }
        Err(Error::InvalidHost)
    }
}

impl Host {
    /// A convenient method to simply check if a host is an IP address
    pub fn is_ip(&self) -> bool {
        if let Host::Ip(_) = self {
            return true;
        }
        false
    }

    /// A convenient method to simply check if a host is a domain name
    pub fn is_domain(&self) -> bool {
        if let Host::Domain(_) = self {
            return true;
        }
        false
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Host::Ip(ip) => write!(f, "{}", ip),
            Host::Domain(domain) => write!(f, "{}", domain),
        }
    }
}

#[allow(clippy::cmp_owned)]
impl PartialEq<str> for Host {
    fn eq(&self, other: &str) -> bool {
        self.to_string() == other
    }
}
