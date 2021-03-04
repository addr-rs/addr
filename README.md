# Robust and fast domain name parsing

[![Build Status](https://travis-ci.org/addr-rs/addr.svg?branch=main)](https://travis-ci.org/addr-rs/addr) [![Latest Version](https://img.shields.io/crates/v/addr.svg)](https://crates.io/crates/addr) [![Docs](https://docs.rs/addr/badge.svg)](https://docs.rs/addr)

This library uses Mozilla's [Public Suffix List](https://publicsuffix.org) to reliably parse domain names in [Rust](https://www.rust-lang.org). It will reliably check if a domain has valid syntax. It also checks the length restrictions for each label, total number of labels and full length of domain name.

## Examples

```rust
use addr::{dns, domain};

fn main() -> addr::Result<()> {
    // You can find out the root domain
    // or extension of any given domain name
    let domain = domain::Name::parse("www.example.com")?;
    assert_eq!(domain.root(), "example.com");
    assert_eq!(domain.suffix(), "com");

    let domain = domain::Name::parse("www.食狮.中国")?;
    assert_eq!(domain.root(), "xn--85x722f.xn--fiqs8s");
    assert_eq!(domain.suffix(), "xn--fiqs8s");

    let domain = domain::Name::parse("www.xn--85x722f.xn--55qx5d.cn")?;
    assert_eq!(domain.root(), "xn--85x722f.xn--55qx5d.cn");
    assert_eq!(domain.suffix(), "xn--55qx5d.cn");

    let domain = domain::Name::parse("a.b.example.uk.com")?;
    assert_eq!(domain.root(), "example.uk.com");
    assert_eq!(domain.suffix(), "uk.com");

    let name = dns::Name::parse("_tcp.example.com.")?;
    assert_eq!(name.root(), Some("example.com."));
    assert_eq!(name.suffix(), Some("com."));

    // In any case if the domain's suffix is in the list
    // then this is definately a registrable domain name
    assert!(domain.suffix_is_known());
}
```

## Use Cases

For those who work with domain names the use cases of this library are plenty. [publicsuffix.org/learn](https://publicsuffix.org/learn/) lists quite a few. For the sake of brevity, I'm not going to repeat them here. I work for a domain registrar so we make good use of this library. Here are some of the ways this library can be used:

* Validating domain names. This one is probably obvious. If a `domain.suffix_is_known()` you can be absolutely sure this is a valid domain name. A regular expression is simply not robust enough.
* Blacklisting or whitelisting domain names. You can't just blindly do this without knowing the actual registrable domain name otherwise you risk being too restrictive or too lenient. Bad news either way...
* Extracting the registrable part of a domain name so you can check whether the domain is registered or not.
* Storing details about a domain name in a DBMS using the registrable part of a domain name as the primary key.
