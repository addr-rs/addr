# A native Rust library for Mozilla's Public Suffix List

```rust
let suffix = psl::suffix(b"www.example.com")?;
assert_eq!(suffix.as_bytes(), b"com");

let domain = psl::domain(b"www.example.com")?;
assert_eq!(domain.as_bytes(), b"example.com");
```
