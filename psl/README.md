# A native Rust library for Mozilla's Public Suffix List

```rust
extern crate psl;

use psl::{Psl, List};

let list = List::new();

let suffix = list.suffix("example.com")?;

let domain = list.domain("example.com")?;
```
