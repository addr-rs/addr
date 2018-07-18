# Robust domain name parsing and RFC compliant email address validation

[![Build Status](https://travis-ci.org/addr-rs/addr.svg?branch=master)](https://travis-ci.org/addr-rs/addr) [![Latest Version](https://img.shields.io/crates/v/addr.svg)](https://crates.io/crates/addr) [![Docs](https://docs.rs/addr/badge.svg)](https://docs.rs/addr)

This library uses Mozilla's [Public Suffix List](https://publicsuffix.org) to reliably parse domain names and email addresses in [Rust](https://www.rust-lang.org). It will reliably check if a domain has valid syntax whether or not it is an internationalised domain name (IDN). It also checks the length restrictions for each label, total number of labels and full length of domain name.

You can supply your own list by setting the environment variable `PSL_URL` or `PSL_PATH`. You can also use the plural forms of those environment variables with the values separated by commas. The first successful list retrieved will be used. If you don't supply your own list, one will be downloaded for you from the official site during the build. If you are only interested in a few TLDs, you can pass them in as `PSL_TLD` or `PSL_TLDS`.

## Examples

```rust
extern crate addr;

use addr::{DomainName, DnsName, Email, Result};

fn main() -> Result<()> {
    // You can find out the root domain
    // or extension of any given domain name
    let domain: DomainName = "www.example.com".parse()?;
    assert_eq!(domain.root(), "example.com.");
    assert_eq!(domain.root().suffix(), "com.");

    let domain: DomainName = "www.食狮.中国".parse()?;
    assert_eq!(domain.root(), "食狮.中国.");
    assert_eq!(domain.root().suffix(), "中国.");

    let domain: DomainName = "www.xn--85x722f.xn--55qx5d.cn".parse()?;
    assert_eq!(domain.root(), "公司.cn.");
    assert_eq!(domain.root().suffix(), "cn.");

    let domain: DomainName = "a.b.example.uk.com".parse()?;
    assert_eq!(domain.root(), "example.uk.com.");
    assert_eq!(domain.root().suffix(), "uk.com.");

    let name: DnsName = "_tcp.example.com.".parse()?;
    assert_eq!(name.root(), "example.com.");
    assert_eq!(name.root().suffix(), "com.");

    let email: Email = "чебурашка@ящик-с-апельсинами.рф".parse()?;
    assert_eq!(email.user(), "чебурашка");
    assert_eq!(email.host(), "ящик-с-апельсинами.рф.");

    // In any case if the domain's suffix is in the list
    // then this is definately a registrable domain name
    assert!(domain.root().suffix().is_known());
    Ok(())
}
```

## Use Cases

For those who work with domain names the use cases of this library are plenty. [publicsuffix.org/learn](https://publicsuffix.org/learn/) lists quite a few. For the sake of brevity, I'm not going to repeat them here. I work for a domain registrar so we make good use of this library. Here are some of the ways this library can be used:-

* Validating domain names. This one is probably obvious. If a `domain.root().suffix().is_known()` you can be absolutely sure this is a valid domain name. A regular expression is simply not robust enough.
* Validating email addresses. You can utilise this library to validate email addresses in a robust and reliable manner before resorting to more expensive (DNS checks) or less convenient (sending confirmation emails) ways.
* Blacklisting or whitelisting domain names and email addresses. You can't just blindly do this without knowing the actual registrable domain name otherwise you risk being too restrictive or too lenient. Bad news either way...
* Extracting the registrable part of a domain name so you can check whether the domain is registered or not.
* Storing details about a domain name in a DBMS using the registrable part of a domain name as the primary key.
* Like my company, a registrar or similar organisation can draft their own list of domain extensions they support, following the same specs as the original list, and then use this library to check whether a requested domain name is actually supported.
