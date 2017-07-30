# A cache manager for the publicsuffix crate

[![Build Status](https://travis-ci.org/rushmorem/psl.svg?branch=master)](https://travis-ci.org/rushmorem/psl) [![Latest Version](https://img.shields.io/crates/v/psl.svg)](https://crates.io/crates/psl) [![Docs](https://docs.rs/psl/badge.svg)](https://docs.rs/psl)

The [publicsuffix](https://github.com/rushmorem/publicsuffix) crate is a low level crate that allows you to optionally use Mozilla's Public Suffix List to parse domain names and email addresses. It provides methods for fetching the list but it doesn't try to handle caching for you. This is where this crate comes in. It downloads the list at a specific interval (once every week by default). When it downloads a new version it saves it to disk. After that, it won't try to download again until the saved one has expired.

## Quick Start

Add this crate to your dependencies:-

```toml
[dependencies]
psl = "0.0.3"
publicsuffix = "1.4"

# The following crates are optional but recommended. Without logging,
# you won't know if updates start failing in future.
slog = "2.0"
slog-term = "2.0"
slog-async = "2.0"
```

Call `init` from your `main.rs`:-

```rust
extern crate psl;
extern crate publicsuffix;
extern crate slog;
extern crate slog_term;
extern crate slog_async;

use publicsuffix::LIST_URL;
use slog::{Logger, Drain};

fn main() {
  // Set up logging
  let decorator = slog_term::TermDecorator::new().build();
  let drain = slog_term::FullFormat::new(decorator).build().fuse();
  let drain = slog_async::Async::new(drain).build().fuse();
  let log = Logger::root(drain, o!("version" => env!("CARGO_PKG_VERSION")));

  // Initialise the list
  psl::init(LIST_URL, None, log).unwrap();
}

fn anywhere() -> Result<()> {
  // To get access to an instance of the cached list call `psl::get()`. For example...
  let domain = psl::get().parse_domain("example.com")?;
}
```
