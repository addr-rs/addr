use crate::{matcher, Error, Result};
use core::fmt;
use psl::{List, Psl};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Name<'a> {
    full: &'a str,
    suffix: psl::Suffix<'a>,
}

impl<'a> Name<'a> {
    pub fn parse(name: &'a str) -> Result<Name<'a>> {
        let stripped = name.strip_suffix('.').unwrap_or(name);
        if stripped.contains('.') {
            matcher::is_domain_name(stripped)?;
        } else {
            matcher::is_label(stripped, true)?;
        }
        Ok(Self {
            suffix: List.suffix(name.as_bytes()).ok_or(Error::InvalidDomain)?,
            full: name,
        })
    }

    pub const fn as_str(&self) -> &str {
        &self.full
    }

    pub fn root(&self) -> Option<&str> {
        let suffix = self.suffix();
        let offset = self
            .full
            .strip_suffix(suffix)?
            .strip_suffix('.')?
            .rfind('.')
            .map(|x| x + 1)
            .unwrap_or(0);
        self.full.get(offset..)
    }

    pub fn suffix(&self) -> &str {
        let offset = self.full.len() - self.suffix.as_bytes().len();
        &self.full[offset..]
    }

    pub const fn has_known_suffix(&self) -> bool {
        self.suffix.is_known()
    }

    pub const fn is_icann(&self) -> bool {
        matches!(self.suffix.typ(), Some(psl::Type::Icann))
    }

    pub const fn is_private(&self) -> bool {
        matches!(self.suffix.typ(), Some(psl::Type::Private))
    }
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.full)
    }
}
