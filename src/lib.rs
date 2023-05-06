//! Yet another zlib implementation.
//!
//! This crate is an implementation of the RFC 1950 DEFLATE specification with
//! support for the zlib wrapper. There are many fine options for such in the
//! Rust ecosystem, but I was looking for one that was small and relatively
//! simple with reasonable performance/compression ratio and support for heap-free
//! compression/decompression scenarios. This crate aims to tick those boxes
//! while also providing composable streaming support based on the standard I/O
//! mechanisms.
//!
//! See the quick start guide below for basic usage or jump to the [compression](#compression)
//! or [decompression](#decompression) section for more detail.
//!
//! # Quick Start
//!
//! So you've got some bytes, they all fit in memory, you don't need to reuse allocations,
//! and you just want to compress or decompress them. This section is for you.
//!
//! Cargo.toml:
//! ```toml
//! [dependencies]
//! yazi = "0.1.4"
//! ```
//!
//! The [`compress`](fn.compress.html) and [`decompress`](fn.decompress.html) functions
//! are provided for the most common use cases:
//! ```
//! use yazi::*;
//! // Your source data.
//! let data = &(0..=255).cycle().take(8192).collect::<Vec<u8>>()[..];
//! // Compress it into a Vec<u8> with a zlib wrapper using the default compression level.
//! let compressed = compress(data, Format::Zlib, CompressionLevel::Default).unwrap();
//! // Decompress it into a Vec<u8>.
//! let (decompressed, checksum) = decompress(&compressed, Format::Zlib).unwrap();
//! // Verify the checksum.
//! assert_eq!(Adler32::from_buf(&decompressed).finish(), checksum.unwrap());
//! // Verify that the decompressed data matches the original.
//! assert_eq!(&decompressed[..], data);
//! ```
//!
//! Read on for more detailed usage.
//!
//! # Compression
//!
//! To compress data, you'll need to create an instance of the
//! [`Encoder`](struct.Encoder.html) struct. The [`new`](struct.Encoder.html#method.new)
//! method can
//! be used to construct an encoder on the stack, but the internal buffers are large
//! (~300k) and may cause a stack overflow so it is advisable to use the
//! [`boxed`](struct.Encoder.html#method.boxed) method to allocate the encoder on the heap.
//!
//! Newly constructed encoders are configured to output a raw DEFLATE bitstream using a
//! medium compression level and a default strategy. Call
//! [`set_format`](struct.Encoder.html#method.set_format) to change the output
//! [`Format`](enum.Format.html). Raw DEFLATE and zlib are supported. The
//! [`set_level`](struct.Encoder.html#method.set_level) method allows you to choose the
//! preferred [`CompressionLevel`](enum.CompressionLevel.html) from a set of basic
//! options or a specific level between 1 and 10. The
//! [`CompressionStrategy`](enum.CompressionStrategy.html) can be changed with the
//! [`set_strategy`](struct.Encoder.html#method.set_strategy) method. This allows you
//! to, for example, force the encoder to output only static blocks.
//!
//! To create an encoder that outputs a zlib bitstream and spends some extra time to potentially
//! produce a result with a higher compression ratio:
//! ```
//! use yazi::{CompressionLevel, Encoder, Format};
//! let mut encoder = Encoder::boxed();
//! encoder.set_format(Format::Zlib);
//! encoder.set_level(CompressionLevel::BestSize);
//! ```
//!
//! The encoder itself does not provide any functionality. It simply stores state and
//! configuration. To actually compress data, you'll need an
//! [`EncoderStream`](struct.EncoderStream.html). A stream is a binding between an
//! encoder and some specific output that will receive the compressed data. This
//! design allows an encoder to be reused with different types of outputs without paying the
//! allocation and initialization cost each time.
//!
//! Streaming supports outputs of the following forms:
//! - Fixed buffers, created with the [`stream_into_buf`](struct.Encoder.html#method.stream_into_buf) method.
//! - Vectors, created with the [`stream_into_vec`](struct.Encoder.html#method.stream_into_vec) method.
//! - Any type that implements [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html),
//!     created with the generic [`stream`](struct.Encoder.html#method.stream) method.
//!
//! Once you have an [`EncoderStream`](struct.EncoderStream.html), simply call
//! [`write`](struct.EncoderStream.html#method.write) one or more times, feeding your raw
//! data into the stream. If available, you can submit the entire input buffer at once, or
//! in arbitrarily sized chunks down to a single byte. After all data has been written,
//! call [`finish`](struct.EncoderStream.html#method.finish) on the stream which will
//! consume it, flush all remaining input and output, and finalize the operation. The finish
//! method returns a [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
//! containing the total number of compressed bytes written to the output on success, or an
//! [`Error`](enum.Error.html) describing the problem on failure.
//!
//! Let's write a function that compresses some arbitrary bytes into a vector:
//! ```
//! fn compress_bytes(buf: &[u8]) -> Result<Vec<u8>, yazi::Error> {
//!     use yazi::Encoder;
//!     let mut encoder = Encoder::boxed();
//!     let mut vec = Vec::new();
//!     let mut stream = encoder.stream_into_vec(&mut vec);
//!     stream.write(buf)?;
//!     stream.finish()?;
//!     Ok(vec)
//! }
//! ```
//!
//! Now let's do something a bit more interesting, and given two paths, compress
//! one file into another:
//! ```
//! fn compress_file(source: &str, dest: &str) -> Result<u64, yazi::Error> {
//!     use yazi::Encoder;
//!     use std::fs::File;
//!     use std::io::{copy, BufWriter};
//!     let mut encoder = Encoder::boxed();
//!     // yazi does not perform any internal buffering beyond what is necessary
//!     // for correctness.
//!     let mut target = BufWriter::new(File::create(dest)?);
//!     let mut stream = encoder.stream(&mut target);
//!     copy(&mut File::open(source)?, &mut stream)?;
//!     stream.finish()
//! }
//! ```
//!
//! Here, we can see that [`EncoderStream`](struct.EncoderStream.html) also implements
//! [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html), so we can
//! pass it directly to [`std::io::copy`](https://doc.rust-lang.org/std/io/fn.copy.html).
//! This allows streams to be composable with the standard I/O facilities and other
//! libraries that support those interfaces.
//!
//! # Decompression
//!
//! If you've already read the section on compression, the API for decompression
//! is essentially identical with the types replaced by [`Decoder`](struct.Decoder.html)
//! and [`DecoderStream`](struct.DecoderStream.html). The documentation is copied here
//! almost verbatim for the sake of completeness and for those who might have skipped
//! directly to this section.
//!
//! To decompress data, you'll need to create an instance of the
//! [`Decoder`](struct.Decoder.html) struct. The [`new`](struct.Decoder.html#method.new)
//! method can be used to construct a decoder on the stack, and unlike encoders, the
//! decoder struct is relatively small (~10k) and generally safe to stack allocate. You can
//! create a decoder on the heap with the [`boxed`](struct.Decoder.html#method.boxed)
//! method if you prefer.
//!
//! Newly constructed decoders are configured to decompress a raw DEFLATE bitstream. Call
//! [`set_format`](struct.Decoder.html#method.set_format) to change the input
//! [`Format`](enum.Format.html). Raw DEFLATE and zlib are supported. No other configuration
//! is necessary for decompression.
//!
//! To create a decoder that decompresses a zlib bitstream:
//! ```
//! use yazi::{Decoder, Format};
//! let mut decoder = Decoder::new();
//! decoder.set_format(Format::Zlib);
//! ```
//!
//! The decoder itself does not provide any functionality. It simply stores state and
//! configuration. To actually decompress data, you'll need a
//! [`DecoderStream`](struct.DecoderStream.html). A stream is a binding between a
//! decoder and some specific output that will receive the decompressed data. This
//! design allows a decoder to be reused with different types of outputs without paying the
//! allocation and initialization cost each time.
//!
//! Streaming supports outputs of the following forms:
//! - Fixed buffers, created with the [`stream_into_buf`](struct.Decoder.html#method.stream_into_buf) method.
//! - Vectors, created with the [`stream_into_vec`](struct.Decoder.html#method.stream_into_vec) method.
//! - Any type that implements [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html),
//!     created with the generic [`stream`](struct.Decoder.html#method.stream) method.
//!
//! Once you have a [`DecoderStream`](struct.DecoderStream.html), simply call
//! [`write`](struct.DecoderStream.html#method.write) one or more times, feeding your compressed
//! data into the stream. If available, you can submit the entire input buffer at once, or
//! in arbitrarily sized chunks down to a single byte. After all data has been written,
//! call [`finish`](struct.DecoderStream.html#method.finish) on the stream which will
//! consume it, flush all remaining input and output, and finalize the operation. The finish
//! method returns a [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
//! containing the total number of decompressed bytes written to the output along with an optional
//! Adler-32 checksum (if the stream was zlib-encoded) on success, or an
//! [`Error`](enum.Error.html) describing the problem on failure.
//!
//! Let's write a function that decompresses a zlib bitstream into a vector and verifies
//! the checksum:
//! ```
//! fn decompress_zlib(buf: &[u8]) -> Result<Vec<u8>, yazi::Error> {
//!     use yazi::{Adler32, Decoder, Error, Format};
//!     let mut decoder = Decoder::new();
//!     decoder.set_format(Format::Zlib);
//!     let mut vec = Vec::new();
//!     let mut stream = decoder.stream_into_vec(&mut vec);
//!     stream.write(buf)?;
//!     // checksum is an Option<u32>
//!     let (_, checksum) = stream.finish()?;
//!     if Adler32::from_buf(&vec).finish() != checksum.unwrap() {
//!         return Err(Error::InvalidBitstream);
//!     }
//!     Ok(vec)
//! }
//! ```
//!
//! Now let's do something a bit more interesting, and given two paths, decompress
//! one file into another:
//! ```
//! fn decompress_file(source: &str, dest: &str) -> Result<(u64, Option<u32>), yazi::Error> {
//!     use yazi::Decoder;
//!     use std::fs::File;
//!     use std::io::{copy, BufWriter};
//!     let mut decoder = Decoder::new();
//!     // yazi does not perform any internal buffering beyond what is necessary
//!     // for correctness.
//!     let mut target = BufWriter::new(File::create(dest)?);
//!     let mut stream = decoder.stream(&mut target);
//!     copy(&mut File::open(source)?, &mut stream)?;
//!     stream.finish()
//! }
//! ```
//!
//! Here, we can see that [`DecoderStream`](struct.DecoderStream.html) also implements
//! [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html), so we can
//! pass it directly to [`std::io::copy`](https://doc.rust-lang.org/std/io/fn.copy.html).
//! This allows streams to be composable with the standard I/O facilities and other
//! libraries that support those interfaces.
//!
//! # Implementation Notes
//!
//! The compressor is based heavily on both miniz (<https://github.com/richgel999/miniz>)
//! by Rich Geldreich and miniz_oxide (<https://github.com/Frommi/miniz_oxide>)
//! by Frommi. The available compression levels and strategies are the same and
//! it should produce an identical bitstream for a given set of options. The
//! decompressor is based on the techniques in libdeflate (<https://github.com/ebiggers/libdeflate>)
//! by Eric Biggers.

mod decode;
mod encode;

use std::io;

pub use decode::{decompress, Decoder, DecoderStream};
pub use encode::{compress, CompressionLevel, CompressionStrategy, Encoder, EncoderStream};

/// Defines the format for a compressed bitstream.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Format {
    /// Raw DEFLATE data.
    Raw,
    /// Zlib header with an Adler-32 footer.
    Zlib,
}

/// Errors that may occur during compression or decompression.
#[derive(Debug)]
pub enum Error {
    /// Not enough input was provided.
    Underflow,
    /// The bitstream was corrupt.
    InvalidBitstream,
    /// Output buffer was too small.
    Overflow,
    /// Attempt to write into a finished stream.
    Finished,
    /// A system I/O error.
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Rolling Adler-32 checksum.
#[derive(Copy, Clone)]
pub struct Adler32(u32);

impl Adler32 {
    /// Creates a new checksum initialized to the default value.
    pub fn new() -> Self {
        Self(1)
    }

    /// Creates a checksum from a buffer.
    pub fn from_buf(buf: &[u8]) -> Self {
        let mut checksum = Self::new();
        checksum.update(buf);
        checksum
    }

    /// Updates the checksum with bytes provided by the specified buffer.
    pub fn update(&mut self, buf: &[u8]) {
        let mut s1 = self.0 & 0xFFFF;
        let mut s2 = (self.0 >> 16) & 0xFFFF;
        for chunk in buf.chunks(5550) {
            for b in chunk {
                s1 += *b as u32;
                s2 += s1;
            }
            s1 %= 65521;
            s2 %= 65521;
        }
        self.0 = (s2 << 16) | s1;
    }

    /// Returns the checksum.
    pub fn finish(self) -> u32 {
        self.0
    }
}

impl Default for Adler32 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_bytes() -> Vec<u8> {
        const BYTES: &[u8; 26] = b"abcdefghijklmnopqrstuvwxyz";
        let mut buf = Vec::new();
        for i in 0..4096 {
            if i % 3 == 0 {
                buf.extend_from_slice(&BYTES[13..]);
            } else if i & 1 != 0 {
                buf.extend_from_slice(BYTES);
            } else {
                buf.extend(BYTES.iter().rev());
            }
        }
        buf
    }

    #[test]
    fn compress_decompress() {
        let buf = generate_bytes();
        let mut compressed = Vec::new();
        let mut encoder = Encoder::boxed();
        let mut stream = encoder.stream_into_vec(&mut compressed);
        stream.write(&buf).unwrap();
        stream.finish().unwrap();
        let mut decompressed = Vec::new();
        let mut decoder = Decoder::new();
        let mut stream = decoder.stream_into_vec(&mut decompressed);
        stream.write(&compressed).unwrap();
        stream.finish().unwrap();
        assert_eq!(buf, decompressed);
    }

    #[test]
    fn compress_decompress_zlib() {
        let buf = generate_bytes();
        let mut compressed = Vec::new();
        let mut encoder = Encoder::boxed();
        encoder.set_format(Format::Zlib);
        let mut stream = encoder.stream_into_vec(&mut compressed);
        stream.write(&buf).unwrap();
        stream.finish().unwrap();
        let mut decompressed = Vec::new();
        let mut decoder = Decoder::new();
        decoder.set_format(Format::Zlib);
        let mut stream = decoder.stream_into_vec(&mut decompressed);
        stream.write(&compressed).unwrap();
        let (_, checksum) = stream.finish().unwrap();
        assert_eq!(buf, decompressed);
        let mut adler = Adler32::new();
        adler.update(&decompressed);
        assert_eq!(adler.finish(), checksum.unwrap());
    }

    #[test]
    fn compress_decompress_static() {
        let buf = generate_bytes();
        let mut compressed = Vec::new();
        let mut encoder = Encoder::boxed();
        encoder.set_strategy(CompressionStrategy::Static);
        let mut stream = encoder.stream_into_vec(&mut compressed);
        stream.write(&buf).unwrap();
        stream.finish().unwrap();
        let mut decompressed = Vec::new();
        let mut decoder = Decoder::new();
        let mut stream = decoder.stream_into_vec(&mut decompressed);
        stream.write(&compressed).unwrap();
        stream.finish().unwrap();
        assert_eq!(buf, decompressed);
    }

    #[test]
    fn compress_decompress_raw() {
        let buf = generate_bytes();
        let mut compressed = Vec::new();
        let mut encoder = Encoder::boxed();
        encoder.set_level(CompressionLevel::None);
        let mut stream = encoder.stream_into_vec(&mut compressed);
        stream.write(&buf).unwrap();
        stream.finish().unwrap();
        let mut decompressed = Vec::new();
        let mut decoder = Decoder::new();
        let mut stream = decoder.stream_into_vec(&mut decompressed);
        stream.write(&compressed).unwrap();
        stream.finish().unwrap();
        assert_eq!(buf, decompressed);
    }

    #[test]
    fn compress_decompress_streaming_1byte() {
        let buf = generate_bytes();
        let mut compressed = Vec::new();
        let mut encoder = Encoder::boxed();
        let mut stream = encoder.stream_into_vec(&mut compressed);
        for &b in &buf {
            stream.write(&[b]).unwrap();
        }
        stream.finish().unwrap();
        let mut decompressed = Vec::new();
        let mut decoder = Decoder::new();
        let mut stream = decoder.stream_into_vec(&mut decompressed);
        for &b in &compressed {
            stream.write(&[b]).unwrap();
        }
        stream.finish().unwrap();
        assert_eq!(buf, decompressed);
    }
    #[test]
    fn compress_decompress_streaming_64bytes() {
        let buf = generate_bytes();
        let mut compressed = Vec::new();
        let mut encoder = Encoder::boxed();
        let mut stream = encoder.stream_into_vec(&mut compressed);
        for chunk in buf.chunks(64) {
            stream.write(chunk).unwrap();
        }
        stream.finish().unwrap();
        let mut decompressed = Vec::new();
        let mut decoder = Decoder::new();
        let mut stream = decoder.stream_into_vec(&mut decompressed);
        for chunk in compressed.chunks(64) {
            stream.write(chunk).unwrap();
        }
        stream.finish().unwrap();
        assert_eq!(buf, decompressed);
    }
}
