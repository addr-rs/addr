use crate::{matcher, Result};
use core::{fmt, str};
use psl_types::{List, Suffix, Type};

/// Holds information about a particular DNS name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Name<'a> {
    full: &'a str,
    suffix: Option<Suffix<'a>>,
}

impl<'a> Name<'a> {
    pub(crate) fn parse<T: List<'a> + ?Sized>(list: &T, name: &'a str) -> Result<Name<'a>> {
        matcher::is_dns_name(name)?;
        Ok(Self {
            full: name,
            suffix: list.suffix(name.as_bytes()),
        })
    }

    pub const fn as_str(&self) -> &str {
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

    pub const fn has_known_suffix(&self) -> bool {
        if let Some(suffix) = self.suffix {
            suffix.is_known()
        } else {
            false
        }
    }

    pub const fn is_icann(&self) -> bool {
        if let Some(suffix) = self.suffix {
            matches!(suffix.typ(), Some(Type::Icann))
        } else {
            false
        }
    }

    pub const fn is_private(&self) -> bool {
        if let Some(suffix) = self.suffix {
            matches!(suffix.typ(), Some(Type::Private))
        } else {
            false
        }
    }
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.full)
    }
}
