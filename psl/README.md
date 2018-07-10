# A native Rust library for Mozilla's Public Suffix List

[![Build Status](https://travis-ci.org/rushmorem/psl.svg?branch=master)](https://travis-ci.org/rushmorem/psl) [![Latest Version](https://img.shields.io/crates/v/psl.svg)](https://crates.io/crates/psl) [![Docs](https://docs.rs/psl/badge.svg)](https://docs.rs/psl)

This crate will eventually supersede the `publicsuffix` crate.

## Setting Up

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
psl = "0.1"
```

## Examples

```rust
extern crate psl;

use psl::{Psl, List};

let list = List::new();

let suffix = list.suffix("example.com")?;

let domain = list.domain("example.com")?;
```

## Use Cases

For those who work with domain names the use cases of this library are plenty. [publicsuffix.org/learn](https://publicsuffix.org/learn/) lists quite a few. For the sake of brevity, I'm not going to repeat them here. I work for a domain registrar so we make good use of this library. Here are some of the ways this library can be used:-

* Validating domain names. This one is probably obvious.
* Validating email addresses.
* Blacklisting or whitelisting domain names and email addresses. You can't just blindly do this without knowing the actual registrable domain name otherwise you risk being too restrictive or too lenient. Bad news either way...
* Extracting the registrable part of a domain name so you can check whether the domain is registered or not.
* Storing details about a domain name in a DBMS using the registrable part of a domain name as the primary key.
