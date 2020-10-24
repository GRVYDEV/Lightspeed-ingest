use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};
enum Command {
    HMAC,
    Connect,
    Ping,
    Dot,
    Unsupported,
}
pub struct FtlCodec {
    delimiter_chars_read: usize,
    command_buffer: std::vec::Vec<u8>,
    bytes_read: usize,
}

impl FtlCodec {
    pub fn new(bytes_read: usize) -> FtlCodec {
        FtlCodec {
            delimiter_chars_read: 0,
            command_buffer: Vec::new(),
            bytes_read,
        }
    }
}
