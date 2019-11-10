use flate2::write::{GzDecoder, GzEncoder};
use flate2::Compression;
use std::io::prelude::*;

// TODO: These should really return Result's and do proper error handling

pub fn gzip_encode(buffer: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());

    encoder
        .write_all(&buffer)
        .expect("error writing bytes to encoder");

    encoder
        .finish()
        .expect("error in compressing bytes for transmit")
}

pub fn gzip_decode(buffer: &[u8]) -> Vec<u8> {
    let mut decoder = GzDecoder::new(Vec::new());

    decoder
        .write_all(&buffer[..])
        .expect("error writing incoming packet to decoder");

    decoder.finish().expect("error decoding incoming packet")
}
