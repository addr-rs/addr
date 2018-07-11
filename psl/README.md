# A native Rust library for Mozilla's Public Suffix List

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
