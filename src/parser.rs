use crate::{Error, Result};

static MAX_DOMAIN_LEN: usize = 253;
static MAX_LABELS_COUNT: usize = 127;
static MAX_LABEL_LEN: usize = 63;

/// Check if a domain has valid syntax
// https://en.wikipedia.org/wiki/Domain_name#Domain_name_syntax
// http://blog.sacaluta.com/2011/12/dns-domain-names-253-or-255-bytesoctets.html
// https://blogs.msdn.microsoft.com/oldnewthing/20120412-00/?p=7873/
#[inline]
pub fn parse_domain(punycode: &str) -> Result<()> {
    if !punycode.is_ascii() {
        return Err(Error::DomainNotAscii);
    }

    let domain = punycode.strip_suffix('.').unwrap_or(punycode);

    // check total lengths
    if domain.len() > MAX_DOMAIN_LEN {
        return Err(Error::DomainTooLong);
    }

    let dot_count = domain.matches('.').count();

    if dot_count + 1 > MAX_LABELS_COUNT {
        return Err(Error::TooManyLabels);
    }

    for (i, label) in domain.split('.').enumerate() {
        let len = label.len();

        if label.trim().is_empty() {
            return Err(Error::EmptyLabel);
        }

        if len > MAX_LABEL_LEN {
            return Err(Error::LabelTooLong);
        }

        if i == dot_count && label.parse::<f64>().is_ok() {
            return Err(Error::NumericTld);
        }

        if !label.starts_with(char::is_alphanumeric) {
            return Err(Error::LabelStartNotAlnum);
        }

        if !label.ends_with(char::is_alphanumeric) {
            return Err(Error::LabelEndNotAlnum);
        }

        if label.contains(|c: char| c != '-' && !c.is_alphanumeric()) {
            return Err(Error::IllegalCharacter);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_label_domain() {
        assert!(parse_domain("xn--example").is_ok());
    }

    #[test]
    fn plain_domain() {
        assert!(parse_domain("example.com").is_ok());
    }

    #[test]
    fn fqdn() {
        assert!(parse_domain("example.com.").is_ok());
    }

    #[test]
    fn subdomains() {
        assert!(parse_domain("a.b.c.d.e.f.").is_ok());
    }
}
