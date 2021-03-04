use crate::parser::parse_dns;
use crate::Result;
use core::{fmt, str};
use psl::Suffix;

/// Holds information about a particular DNS name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Name<'a> {
    full: &'a str,
    suffix: Option<Suffix<'a>>,
}

impl<'a> Name<'a> {
    pub fn parse(name: &'a str) -> Result<Name<'a>> {
        parse_dns(name)?;
        Ok(Self {
            full: name,
            suffix: psl::suffix(name.as_bytes()),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.full
    }

    pub fn root(&self) -> Option<&str> {
        let suffix = self.suffix()?;
        let offset = self
            .full
            .strip_suffix(suffix)?
            .strip_suffix('.')?
            .rfind('.')
            .map(|x| x + 1)
            .unwrap_or(0);
        self.full.get(offset..)
    }

    pub fn suffix(&self) -> Option<&str> {
        let bytes = self.suffix.as_ref()?.as_bytes();
        str::from_utf8(bytes).ok()
    }

    pub fn has_known_suffix(&self) -> bool {
        self.suffix.as_ref().map(Suffix::is_known).unwrap_or(false)
    }

    pub fn is_icann(&self) -> bool {
        self.suffix
            .as_ref()
            .map(Suffix::typ)
            .flatten()
            .filter(|t| *t == psl::Type::Icann)
            .is_some()
    }

    pub fn is_private(&self) -> bool {
        self.suffix
            .as_ref()
            .map(Suffix::typ)
            .flatten()
            .filter(|t| *t == psl::Type::Private)
            .is_some()
    }
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.full)
    }
}
