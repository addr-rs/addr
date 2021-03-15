//! Parser traits

#[cfg(any(feature = "net", feature = "serde-net"))]
use crate::email;
use crate::{dns, domain, Result};
use psl_types::List;

/// Parses a domain using the list
pub trait DomainName<'a> {
    fn parse_domain_name(&self, name: &'a str) -> Result<'a, domain::Name<'a>>;
}

impl<'a, T> DomainName<'a> for T
where
    T: List<'a>,
{
    fn parse_domain_name(&self, name: &'a str) -> Result<'a, domain::Name<'a>> {
        domain::Name::parse(self, name).map_err(|kind| kind.error_with(name))
    }
}

/// Parses any arbitrary string that can be used as a key in a DNS database
pub trait DnsName<'a> {
    fn parse_dns_name(&self, name: &'a str) -> Result<'a, dns::Name<'a>>;
}

impl<'a, T> DnsName<'a> for T
where
    T: List<'a>,
{
    fn parse_dns_name(&self, name: &'a str) -> Result<'a, dns::Name<'a>> {
        dns::Name::parse(self, name).map_err(|kind| kind.error_with(name))
    }
}

/// Parses an email address using the list
#[cfg(any(feature = "net", feature = "serde-net"))]
pub trait EmailAddress<'a> {
    fn parse_email_address(&self, name: &'a str) -> Result<'a, email::Address<'a>>;
}

#[cfg(any(feature = "net", feature = "serde-net"))]
impl<'a, T> EmailAddress<'a> for T
where
    T: List<'a>,
{
    fn parse_email_address(&self, name: &'a str) -> Result<'a, email::Address<'a>> {
        email::Address::parse(self, name).map_err(|kind| kind.error_with(name))
    }
}
