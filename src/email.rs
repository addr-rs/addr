use crate::domain::Name;
use crate::net::IpAddr;
use crate::{matcher, Error, Result};
use core::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Address<'a> {
    full: &'a str,
    at_sign: usize,
    host: Host<'a>,
}

impl<'a> Address<'a> {
    pub fn parse(address: &'a str) -> Result<Address<'a>> {
        if address.chars().count() > 254 {
            return Err(Error::EmailTooLong);
        }
        let at_sign = address.rfind('@').ok_or(Error::NoAtSign)?;
        let local = address.get(..at_sign).ok_or(Error::NoUserPart)?;
        matcher::is_email_local(local)?;
        let rest = address.get(at_sign + 1..).ok_or(Error::NoHostPart)?;
        let host = Host::parse(rest)?;
        Ok(Self {
            host,
            at_sign,
            full: address,
        })
    }

    pub const fn as_str(&self) -> &str {
        &self.full
    }

    pub const fn host(&self) -> Host<'a> {
        self.host
    }

    pub fn user(&self) -> &str {
        &self.full[..self.at_sign]
    }
}

impl fmt::Display for Address<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.full)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Host<'a> {
    Domain(Name<'a>),
    IpAddr(IpAddr),
}

impl<'a> Host<'a> {
    pub(crate) fn parse(host: &'a str) -> Result<Host<'a>> {
        match host.strip_prefix('[') {
            Some(h) => {
                let ip_addr = h
                    .strip_suffix(']')
                    .ok_or(Error::IllegalCharacter)?
                    .parse()?;
                Ok(Host::IpAddr(ip_addr))
            }
            None => Ok(Host::Domain(Name::parse(host)?)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Address;

    #[test]
    fn parse() {
        // Valid email addresses
        Address::parse("johndoe@example.com").unwrap();
        Address::parse("john.doe@example.com").unwrap();
        Address::parse("john+doe@example.com").unwrap();
        Address::parse(r#""john doe"@example.com"#).unwrap();

        // Invalid email addresses
        Address::parse("@example.com").unwrap_err();
        Address::parse(r#""@example.com"#).unwrap_err();
        Address::parse(" @example.com").unwrap_err();
    }

    #[test]
    fn user() {
        let email = Address::parse("johndoe@localhost").unwrap();
        assert_eq!(email.user(), "johndoe");
    }
}
