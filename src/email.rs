use crate::domain::Name;
use crate::{matcher, Error, Result};
use core::fmt;

pub type User<'a> = &'a str;
pub type Host<'a> = &'a str;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Address<'a> {
    full: &'a str,
    host: Name<'a>,
}

impl<'a> Address<'a> {
    pub fn parse(address: &'a str) -> Result<Address<'a>> {
        if address.chars().count() > 254 {
            return Err(Error::EmailTooLong);
        }
        let (local, host) = split(address)?;
        matcher::is_email_local(local)?;
        Ok(Self {
            host: Name::parse(host)?,
            full: address,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.full
    }

    pub fn host(&self) -> Name<'a> {
        self.host
    }

    pub fn user(&self) -> &str {
        let at_sign = self.full.len() - (self.host.as_str().len() + 1);
        &self.full[..at_sign]
    }
}

pub fn split(address: &str) -> Result<(User, Host)> {
    let at_sign = address.rfind('@').ok_or(Error::NoAtSign)?;
    let local = address.get(..at_sign).ok_or(Error::NoUserPart)?;
    let host = address.get(at_sign + 1..).ok_or(Error::NoHostPart)?;
    Ok((local, host))
}

impl fmt::Display for Address<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.full)
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
    fn split() {
        assert_eq!(super::split("user@localhost"), Ok(("user", "localhost")));
    }

    #[test]
    fn user() {
        let email = Address::parse("johndoe@localhost").unwrap();
        assert_eq!(email.user(), "johndoe");
    }
}
