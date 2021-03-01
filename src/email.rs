use std::fmt;
use std::str::FromStr;

use crate::errors::{Error, ErrorKind, Result};
use crate::{Email, Host};
use regex::RegexSet;

lazy_static::lazy_static! {
    // Regex for matching the local-part of an
    // email address
    static ref LOCAL: RegexSet = {
        // these characters can be anywhere in the expresion
        let global = r#"[[:alnum:]!#$%&'*+/=?^_`{|}~-]"#;
        // non-ascii characters (an also be unquoted)
        let non_ascii = r#"[^\x00-\x7F]"#;
        // the pattern to match
        let quoted = r#"["(),\\:;<>@\[\]. ]"#;
        // combined regex
        let combined = format!(r#"({}+|{}+)"#, global, non_ascii);

        let exprs = vec![
            // can be any combination of allowed characters
            format!(r#"^{}+$"#, combined),
            // can be any combination of allowed charaters
            // separated by a . in between
            format!(r#"^({0}+[.]?{0}+)+$"#, combined),
            // can be a quoted string with allowed plus
            // additional characters
            format!(r#"^"({}*{}*)*"$"#, combined, quoted),
        ];

        RegexSet::new(exprs).unwrap()
    };
}

impl FromStr for Email {
    type Err = Error;

    /// Extracts Host from an email address
    ///
    /// This method can also be used, simply to validate an email address.
    /// If it returns an error, the email address is not valid.
    // https://en.wikipedia.org/wiki/Email_address#Syntax
    // https://en.wikipedia.org/wiki/International_email#Email_addresses
    // http://girders.org/blog/2013/01/31/dont-rfc-validate-email-addresses/
    // https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
    // https://hackernoon.com/the-100-correct-way-to-validate-email-addresses-7c4818f24643#.pgcir4z3e
    // http://haacked.com/archive/2007/08/21/i-knew-how-to-validate-an-email-address-until-i.aspx/
    // https://tools.ietf.org/html/rfc6530#section-10.1
    // http://rumkin.com/software/email/rules.php
    fn from_str(address: &str) -> Result<Email> {
        let mut parts = address.rsplitn(2, '@');
        let host = match parts.next() {
            Some(host) => host,
            None => {
                return Err(ErrorKind::InvalidEmail.into());
            }
        };
        let local = match parts.next() {
            Some(local) => local,
            None => {
                return Err(ErrorKind::InvalidEmail.into());
            }
        };
        if local.chars().count() > 64
            || address.chars().count() > 254
            || (!local.starts_with('"') && local.contains(".."))
            || !LOCAL.is_match(local)
        {
            return Err(ErrorKind::InvalidEmail.into());
        }
        let host = Host::from_str(host)?;
        let name = local.to_owned();
        Ok(Email { name, host })
    }
}

impl Email {
    pub fn user(&self) -> &str {
        &self.name
    }

    pub fn host(&self) -> &Host {
        &self.host
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.host)
    }
}
