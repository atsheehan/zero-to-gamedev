use flate2::write::{GzDecoder, GzEncoder};
use flate2::Compression;
use std::io::prelude::*;

use crate::net::Packet;

// TODO: These should really return Result's and do proper error handling

pub fn gzip_encode(packet: Packet) -> Vec<u8> {
    let packet_bytes = packet.as_bytes();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());

    encoder
        .write_all(&packet_bytes)
        .expect("error writing bytes to encoder");
    let bytes = encoder
        .finish()
        .expect("error in compressing bytes for transmit");

    bytes
}

pub fn gzip_decode(buffer: &[u8]) -> Vec<u8> {
    let mut writer = Vec::new();
    let mut decoder = GzDecoder::new(writer);
    decoder
        .write_all(&buffer[..])
        .expect("error writing incoming packet to decoder");
    writer = decoder.finish().expect("error decoding incoming packet");
    writer
}
