#![no_std]
#![forbid(unsafe_code)]

//! A native Rust library for Mozilla's Public Suffix List

mod list;
#[cfg(feature = "serde")]
mod serde;

/// Get the public suffix of the domain
///
/// *NB:* `name` must be a valid domain name in lowercase
#[inline]
pub fn suffix(name: &[u8]) -> Option<Suffix<'_>> {
    let mut labels = name.rsplit(|x| *x == b'.');
    let fqdn = if name.ends_with(b".") {
        labels.next();
        true
    } else {
        false
    };
    let Info { mut len, typ } = list::lookup(labels);
    if fqdn {
        len += 1;
    }
    if len == 0 {
        return None;
    }
    let offset = name.len() - len;
    let bytes = &name[offset..];
    Some(Suffix { bytes, fqdn, typ })
}

/// Get the registrable domain
///
/// *NB:* `name` must be a valid domain name in lowercase
#[inline]
pub fn domain(name: &[u8]) -> Option<Domain<'_>> {
    let suffix = suffix(name)?;
    let name_len = name.len();
    let suffix_len = suffix.bytes.len();
    if name_len < suffix_len + 2 {
        return None;
    }
    let offset = name_len - (1 + suffix_len);
    let subdomain = &name[..offset];
    let root_label = subdomain.rsplitn(2, |x| *x == b'.').next()?;
    let registrable_len = root_label.len() + 1 + suffix_len;
    let offset = name_len - registrable_len;
    let bytes = &name[offset..];
    Some(Domain { bytes, suffix })
}

/// Type of suffix
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Type {
    Icann,
    Private,
}

/// Information about the suffix
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct Info {
    len: usize,
    typ: Option<Type>,
}

/// The suffix of a domain name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Suffix<'a> {
    bytes: &'a [u8],
    fqdn: bool,
    typ: Option<Type>,
}

impl Suffix<'_> {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn is_fqdn(&self) -> bool {
        self.fqdn
    }

    #[inline]
    pub fn typ(&self) -> Option<Type> {
        self.typ
    }

    #[inline]
    pub fn is_known(&self) -> bool {
        self.typ.is_some()
    }
}

/// A registrable domain name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Domain<'a> {
    bytes: &'a [u8],
    suffix: Suffix<'a>,
}

impl Domain<'_> {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn suffix(&self) -> Suffix<'_> {
        self.suffix
    }
}
