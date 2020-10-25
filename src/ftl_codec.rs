use bytes::{Buf, BufMut, BytesMut};
use std::{fmt, io};
use tokio_util::codec::{Decoder, Encoder};
const COMMAND_DELIMITERS: [char; 4] = ['\r', '\n', '\r', '\n'];
#[derive(Debug)]
pub enum Command {
    HMAC,
    Connect,
    Ping,
    Dot,
    Unsupported,
}
#[derive(Debug)]
pub struct FtlCommand {
    pub command: Command,
    pub data: Option<BytesMut>,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FtlCodec {
    delimiter_chars_read: usize,
    command_buffer: std::vec::Vec<u8>,
}

impl FtlCommand {
    pub fn new(command: Command, data: Option<BytesMut>) -> FtlCommand {
        FtlCommand { command, data }
    }
}
impl FtlCodec {
    pub fn new() -> FtlCodec {
        FtlCodec {
            delimiter_chars_read: 0,
            command_buffer: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.command_buffer = Vec::new();
        self.delimiter_chars_read = 0;
    }
}

impl Decoder for FtlCodec {
    type Item = FtlCommand;
    type Error = FtlError;
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<FtlCommand>, FtlError> {
        match buf.len() {
            0 => Err(FtlError::ConnectionClosed),
            _ => {
                let mut command: String;
                for i in 0..buf.bytes().len() {
                    self.command_buffer.push(buf.bytes()[i]);
                    buf.advance(1);
                    if buf[i] as char == COMMAND_DELIMITERS[self.delimiter_chars_read] {
                        self.delimiter_chars_read += 1;
                        if self.delimiter_chars_read >= COMMAND_DELIMITERS.len() {
                            command = String::from_utf8_lossy(&self.command_buffer.as_slice())
                                .to_string();
                            command.truncate(command.len() - 4);
                            println!("Command is: {:?}", command);
                            match command.as_str() {
                                "HMAC" => {
                                    self.reset();
                                   
                                    return Ok(Some(FtlCommand::new(Command::HMAC, None)));
                                }
                                _ => {
                                    self.reset();
                                    return Err(FtlError::Unsupported(command));
                                }
                            }
                        }
                    }
                }
             
                Err(FtlError::CommandNotFound)
            }
        }
    }
}
impl<T> Encoder<T> for FtlCodec
where
    T: AsRef<str>,
{
    type Error = FtlError;

    fn encode(&mut self, line: T, buf: &mut BytesMut) -> Result<(), FtlError> {
        let line = line.as_ref();
        buf.reserve(line.len());
        buf.put(line.as_bytes());
        Ok(())
    }
}
#[derive(Debug)]
pub enum FtlError {
    ConnectionClosed,
    Unsupported(String),
    CommandNotFound,
    Io(io::Error),
}
impl fmt::Display for FtlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FtlError::ConnectionClosed => write!(f, "Connection Closed"),
            FtlError::CommandNotFound => write!(f, "Command not read"),
            FtlError::Io(e) => write!(f, "{}", e),
            FtlError::Unsupported(s) => {
                write!(f, "Unsupported FTL Command {}! Bug GRVY to support this", s)
            }
        }
    }
}
impl From<io::Error> for FtlError {
    fn from(e: io::Error) -> FtlError {
        FtlError::Io(e)
    }
}
impl std::error::Error for FtlError {}
