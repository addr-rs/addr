use nom::*;

macro_rules! validates {
    ($result:expr) => {{
        match $result {
            Ok((rest, _)) => rest.is_empty(),
            Err(error) => error.is_incomplete(),
        }
    }}
}

named!(label<&str, &str>, alt!(delimited!(alphanumeric, is_a!("-"), alphanumeric) | alphanumeric));

named!(dot_sep<&str, &str>, alt!(delimited!(label, tag!("."), label) | label));

named!(domain_name<&str, &str>, do_parse!(
    domain: dot_sep >>
    opt!(tag!(".")) >>
    (domain)
));

pub fn is_domain(input: &str) -> bool {
    validates!(domain_name(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_label() {
        assert!(validates!(label("example")));
    }

    #[test]
    fn plain_with_dash() {
        assert!(validates!(label("exam-ple")));
    }

    #[test]
    fn plain_domain() {
        assert!(is_domain("example.com"));
    }

    #[test]
    fn fqdn() {
        assert!(is_domain("example.com."));
    }
}
