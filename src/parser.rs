use idna::domain_to_ascii;

static MAX_DOMAIN_LEN: usize = 253;
static MAX_LABELS_COUNT: usize = 127;
static MAX_LABEL_LEN: usize = 63;

#[inline]
fn maybe_tld(input: &str) -> bool {
    !gt_max_label_len(input) && input.parse::<f64>().is_err()
}

#[inline]
fn gt_max_label_len(label: &str) -> bool {
    label.len() > MAX_LABEL_LEN
}

/// Check if a domain has valid syntax
// https://en.wikipedia.org/wiki/Domain_name#Domain_name_syntax
// http://blog.sacaluta.com/2011/12/dns-domain-names-253-or-255-bytesoctets.html
// https://blogs.msdn.microsoft.com/oldnewthing/20120412-00/?p=7873/
#[inline]
pub fn parse_domain(input: &str) -> Result<String, String> {
    let punycode = if input.is_ascii() {
        to_targetcase(input)
    } else if let Ok(punycode) = domain_to_ascii(input) {
        punycode
    } else {
        return Err(to_targetcase(input));
    };
    let is_valid = {
        let punycode = if punycode.ends_with('.') {
            &punycode[..punycode.len()-1]
        } else {
            &punycode
        };
        let mut labels = punycode.rsplit('.');
        // check total lengths
        if punycode.len() > MAX_DOMAIN_LEN || labels.clone().count() > MAX_LABELS_COUNT {
            false
        } else {
            let first_maybe_tld = labels.clone().next().map(maybe_tld);
            // check individual labels
            if first_maybe_tld == Some(false) || first_maybe_tld.is_none() || labels.clone().any(gt_max_label_len) {
                false
            } else {
                let check_labels = || {
                    for label in labels {
                        if label.trim().is_empty() { return false; }
                        let mut chars = label.chars();
                        let last = label.len() - 1;
                        for (i, c) in chars.enumerate() {
                            if ((i == 0 || i == last) && !c.is_alphanumeric()) || (c != '-' && !c.is_alphanumeric()) {
                                return false;
                            }
                        }
                    }
                    true
                };
                check_labels()
            }
        }
    };
    if is_valid {
        Ok(punycode)
    } else {
        Err(punycode)
    }
}

#[inline]
pub fn to_targetcase(input: &str) -> String {
    if cfg!(feature = "anycase") {
        input.to_owned()
    } else {
        input.to_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_tld() {
        assert!(!maybe_tld("1234"));
    }

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
