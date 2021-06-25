# Yazi - Yet another zlib implementation

Yazi is a pure Rust implementation of the RFC 1950 DEFLATE specification with support for
the zlib wrapper. It provides streaming compression and decompression.

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/yazi.svg
[crates-url]: https://crates.io/crates/yazi
[docs-badge]: https://docs.rs/yazi/badge.svg
[docs-url]: https://docs.rs/yazi
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
yazi = "0.1.4"
```

The following demonstrates simple usage for compressing and decompressing in-memory buffers:

```rust
use yazi::*;
// Your source data.
let data = &(0..=255).cycle().take(8192).collect::<Vec<u8>>()[..];
// Compress it into a Vec<u8> with a zlib wrapper using the default compression level.
let compressed = compress(data, Format::Zlib, CompressionLevel::Default).unwrap();
// Decompress it into a Vec<u8>.
let (decompressed, checksum) = decompress(&compressed, Format::Zlib).unwrap();
// Verify the checksum.
assert_eq!(Adler32::from_buf(&decompressed).finish(), checksum.unwrap());
// Verify that the decompressed data matches the original.
assert_eq!(data, &decompressed[..]);
```

For detail on more advanced usage, see the full API [documentation](https://docs.rs/yazi).
