serde_www_form_urlencoded
===================================

# Installation

This crate works with Cargo and can be found on crates.io with a Cargo.toml like:
```toml
[dependencies]
serde_www_form_urlencoded = "0.1.0"
```

The documentation is available on [docs.rs].

[crates.io]: https://crates.io/crates/serde_www_form_urlencoded
[docs.rs]: https://docs.rs/serde_www_form_urlencoded/0.1.0/serde_www_form_urlencoded/

# Format

## Map / struct
Map or struct values are flat-encoded.

## Sequence
Sequence are flat-encoded with a $length attribute to keep track of the number of items.