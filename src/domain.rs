use crate::parser::parse_domain;
use crate::{Error, Result};
use core::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Name<'a> {
    full: &'a str,
    root: psl::Domain<'a>,
}

impl<'a> Name<'a> {
    pub fn parse(name: &'a str) -> Result<Name<'a>> {
        parse_domain(name)?;
        Ok(Self {
            root: psl::domain(name.as_bytes()).ok_or(Error::InvalidDomain)?,
            full: name,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.full
    }

    pub fn root(&self) -> &str {
        let offset = self.full.len() - self.root.as_bytes().len();
        &self.full[offset..]
    }

    pub fn suffix(&self) -> &str {
        let offset = self.full.len() - self.root.suffix().as_bytes().len();
        &self.full[offset..]
    }

    pub fn suffix_is_known(&self) -> bool {
        self.root.suffix().is_known()
    }

    pub fn is_icann(&self) -> bool {
        self.root
            .suffix()
            .typ()
            .filter(|t| *t == psl::Type::Icann)
            .is_some()
    }

    pub fn is_private(&self) -> bool {
        self.root
            .suffix()
            .typ()
            .filter(|t| *t == psl::Type::Private)
            .is_some()
    }
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.full)
    }
}
