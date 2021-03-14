//! The errors returned by this crate

use core::fmt;

pub(crate) type Result<T> = core::result::Result<T, Kind>;

/// Information about the error and its input
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Error<'a> {
    kind: Kind,
    input: &'a str,
}

impl<'a> Error<'a> {
    pub(crate) const fn new(kind: Kind, input: &'a str) -> Self {
        Self { kind, input }
    }

    /// The kind of error this is
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// The input that resulted in this error
    pub const fn input(&self) -> &str {
        self.input
    }
}

/// Description of the error
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[non_exhaustive]
pub enum Kind {
    NameTooLong,
    EmptyLabel,
    EmailLocalTooLong,
    EmailTooLong,
    EmptyName,
    IllegalCharacter,
    InvalidDomain,
    InvalidIpAddr,
    LabelEndNotAlnum,
    LabelStartNotAlnum,
    LabelTooLong,
    NoAtSign,
    NoHostPart,
    NoUserPart,
    NumericTld,
    QuoteUnclosed,
    TooManyLabels,
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO incorporate the input into these to make them more descriptive?
        match self.kind {
            Kind::NameTooLong => write!(f, "name too long"),
            Kind::EmptyLabel => write!(f, "domain/email contains empty label"),
            Kind::EmailLocalTooLong => write!(f, "email local too long"),
            Kind::EmailTooLong => write!(f, "email too long"),
            Kind::EmptyName => write!(f, "name is empty"),
            Kind::IllegalCharacter => write!(f, "domain contains illegal characters"),
            Kind::InvalidDomain => write!(f, "invalid domain name"),
            Kind::InvalidIpAddr => write!(f, "email has an invalid ip address"),
            Kind::LabelEndNotAlnum => {
                write!(f, "label does not start with an alphanumeric character")
            }
            Kind::LabelStartNotAlnum => {
                write!(f, "label does not end with a alphanumeric character")
            }
            Kind::LabelTooLong => write!(f, "label too long"),
            Kind::NoAtSign => write!(f, "email address has no at sign"),
            Kind::NoHostPart => write!(f, "email address has no host part"),
            Kind::NoUserPart => write!(f, "email address has no user part"),
            Kind::NumericTld => write!(f, "numeric TLD"),
            Kind::QuoteUnclosed => write!(f, "email has an unclosed quotation mark"),
            Kind::TooManyLabels => write!(f, "too many labels"),
        }
    }
}

#[cfg(feature = "std")]
impl<'a> std::error::Error for Error<'a> {}
