use failure_derive::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
  #[fail(display = "io error: {}", _0)]
  Io(std::io::Error),
  #[fail(display = "invalid magic bytes")]
  InvalidMagic,
  #[fail(display = "invalid BOM")]
  InvalidBom,
  #[fail(display = "invalid encoding: {}", _0)]
  InvalidEncoding(u8),
  #[fail(display = "invalid utf-8: {}", _0)]
  InvalidUtf8(std::string::FromUtf8Error),
  #[fail(display = "invalid utf-16: {}", _0)]
  InvalidUtf16(std::string::FromUtf16Error),
  #[fail(display = "invalid section header: {:?}", _0)]
  InvalidSection([u8; 4]),
}
