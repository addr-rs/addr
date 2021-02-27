//! Public Suffix List Lexer
//!
//! All this library does is provide methods for fetching and parsing
//! Mozilla's Public Suffix List. You may be interested in higher level
//! libraries like `psl`.
//!
//! ## Examples
//!
//! ```rust,no_run
//! use psl_lexer::List;
//! # use psl_lexer::Result;
//!
//! # fn examples() -> Result<()> {
//! // Fetch the list from the official URL,
//! # #[cfg(feature = "remote_list")]
//! let list = List::fetch()?;
//!
//! // from your own URL
//! # #[cfg(feature = "remote_list")]
//! let list = List::from_url("https://example.com/path/to/public_suffix_list.dat")?;
//!
//! // or from a local file.
//! let list = List::from_path("/path/to/public_suffix_list.dat")?;
//! # Ok(())
//! # }
//! # fn main() {}
//! ```

pub mod errors;

#[cfg(feature = "remote_list")]
#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::Read;
#[cfg(feature = "remote_list")]
use std::io::Write;
#[cfg(feature = "remote_list")]
use std::net::TcpStream;
use std::path::Path;
use std::str;
use std::str::FromStr;
#[cfg(feature = "remote_list")]
use std::time::Duration;

pub use errors::{Error, Result};

use errors::ErrorKind;
use indexmap::IndexMap;
#[cfg(feature = "remote_list")]
use native_tls::TlsConnector;
use url::Url;

/// The official URL of the list
pub const LIST_URL: &str = "https://publicsuffix.org/list/public_suffix_list.dat";

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Suffix {
    pub rule: String,
    pub typ: Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Icann,
    Private,
}

/// Stores the public suffix list
///
/// You can use the methods, `fetch`, `from_url` or `from_path` to build the list.
/// If you are using this in a long running server it's recommended you use either
/// `fetch` or `from_url` to download updates at least once a week.
#[derive(Debug)]
pub struct List {
    pub rules: IndexMap<String, Vec<Suffix>>,
}

/// Converts a type into a Url object
pub trait IntoUrl {
    fn into_url(self) -> Result<Url>;
}

impl IntoUrl for Url {
    fn into_url(self) -> Result<Url> {
        Ok(self)
    }
}

impl<'a> IntoUrl for &'a str {
    fn into_url(self) -> Result<Url> {
        Ok(Url::parse(self)?)
    }
}

impl<'a> IntoUrl for &'a String {
    fn into_url(self) -> Result<Url> {
        Ok(Url::parse(self)?)
    }
}

impl IntoUrl for String {
    fn into_url(self) -> Result<Url> {
        Ok(Url::parse(&self)?)
    }
}

#[cfg(feature = "remote_list")]
pub fn request<U: IntoUrl>(u: U) -> Result<String> {
    let url = u.into_url()?;
    let addr = url.socket_addrs(|| None)?;
    let host = match url.host_str() {
        Some(host) => host,
        None => {
            return Err(ErrorKind::NoHost.into());
        }
    };
    let data = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", url.path(), host);
    let stream = TcpStream::connect(&*addr)?;
    let timeout = Duration::from_secs(2);
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    let mut res = String::new();

    match url.scheme() {
        scheme if scheme == "https" => {
            let connector = TlsConnector::builder().build()?;
            let mut stream = connector.connect(host, stream)?;
            stream.write_all(data.as_bytes())?;
            stream.read_to_string(&mut res)?;
        }
        scheme if scheme == "http" => {
            let mut stream = stream;
            stream.write_all(data.as_bytes())?;
            stream.read_to_string(&mut res)?;
        }
        _ => {
            return Err(ErrorKind::UnsupportedScheme.into());
        }
    }

    Ok(res)
}

impl List {
    fn append(&mut self, rule: &str, typ: Type) -> Result<()> {
        rule.rsplit('.')
            .next()
            .ok_or_else(|| ErrorKind::InvalidRule(rule.into()).into())
            .and_then(|tld| {
                if tld.is_empty() {
                    return Err(ErrorKind::InvalidRule(rule.into()).into());
                }
                Ok(tld)
            })
            .map(|tld| {
                self.rules
                    .entry(tld.into())
                    .or_insert_with(Vec::new)
                    .push(Suffix {
                        typ,
                        rule: rule.into(),
                    });
            })
    }

    /// Pull the list from a URL
    #[cfg(feature = "remote_list")]
    pub fn from_url<U: IntoUrl>(url: U) -> Result<List> {
        request(url).and_then(|list| Self::from_str(&list))
    }

    /// Fetch the list from a local file
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<List> {
        File::open(path)
            .map_err(|err| ErrorKind::Io(err).into())
            .and_then(|mut data| {
                let mut res = String::new();
                data.read_to_string(&mut res)?;
                Self::from_str(&res)
            })
    }

    /// Build the list from the result of anything that implements `std::io::Read`
    ///
    /// If you don't already have your list on the filesystem but want to use your
    /// own library to fetch the list you can use this method so you don't have to
    /// save it first.
    pub fn from_reader<R: Read>(mut reader: R) -> Result<List> {
        let mut res = String::new();
        reader.read_to_string(&mut res)?;
        Self::from_str(&res)
    }

    /// Pull the list from the official URL
    #[cfg(feature = "remote_list")]
    pub fn fetch() -> Result<List> {
        let github =
            "https://raw.githubusercontent.com/publicsuffix/list/master/public_suffix_list.dat";

        Self::from_url(LIST_URL)
            // Fallback to the Github repo if the official link
            // is down for some reason.
            .or_else(|_| Self::from_url(github))
    }

    fn find_type(&self, typ: Type) -> Vec<&str> {
        self.rules
            .values()
            .fold(Vec::new(), |mut res, ref suffices| {
                for suffix in *suffices {
                    if suffix.typ == typ {
                        res.push(&suffix.rule);
                    }
                }
                res
            })
    }

    /// Gets a list of all ICANN domain suffices
    pub fn icann(&self) -> Vec<&str> {
        self.find_type(Type::Icann)
    }

    /// Gets a list of all private domain suffices
    pub fn private(&self) -> Vec<&str> {
        self.find_type(Type::Private)
    }

    /// Gets a list of all domain suffices
    pub fn all(&self) -> Vec<&str> {
        self.rules
            .values()
            .fold(Vec::new(), |mut res, ref suffices| {
                for suffix in *suffices {
                    res.push(&suffix.rule);
                }
                res
            })
    }
}

impl FromStr for List {
    type Err = Error;

    fn from_str(res: &str) -> Result<Self> {
        let mut typ = None;
        let mut list = List {
            rules: IndexMap::new(),
        };
        for line in res.lines() {
            match line {
                line if line.contains("BEGIN ICANN DOMAINS") => {
                    typ = Some(Type::Icann);
                }
                line if line.contains("BEGIN PRIVATE DOMAINS") => {
                    typ = Some(Type::Private);
                }
                line if line.starts_with("//") => {
                    continue;
                }
                line => match typ {
                    Some(typ) => {
                        let rule = match line.split_whitespace().next() {
                            Some(rule) => rule,
                            None => continue,
                        };
                        list.append(rule, typ)?;
                    }
                    None => {
                        continue;
                    }
                },
            }
        }
        if list.rules.is_empty() || list.all().is_empty() {
            return Err(ErrorKind::InvalidList.into());
        }
        Ok(list)
    }
}
