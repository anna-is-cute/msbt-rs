pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  Io(std::io::Error),
  InvalidMagic,
  InvalidBom,
  InvalidEncoding(u8),
  InvalidUtf8(std::string::FromUtf8Error),
  InvalidUtf16(std::string::FromUtf16Error),
  InvalidSection([u8; 4]),
}
