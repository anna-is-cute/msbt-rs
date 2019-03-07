use crate::{Msbt, Encoding};
use super::Section;

use byteordered::Endian;

use std::ptr::NonNull;

#[derive(Debug)]
pub struct Txt2 {
  pub(crate) msbt: NonNull<Msbt>,
  pub(crate) section: Section,
  pub(crate) string_count: u32,
  pub(crate) strings: Vec<String>,
  pub(crate) raw_strings: Vec<Vec<u8>>,
}

impl Txt2 {
  pub fn msbt(&self) -> &Msbt {
    unsafe { self.msbt.as_ref() }
  }

  pub fn section(&self) -> &Section {
    &self.section
  }

  pub fn string_count(&self) -> u32 {
    self.string_count
  }

  pub fn strings(&self) -> &[String] {
    &self.strings
  }

  pub fn set_strings<I, S>(&mut self, strings: I)
    where I: IntoIterator<Item = S>,
          S: Into<String>,
  {
    self.strings = strings.into_iter().map(Into::into).collect();
    match self.msbt().header.encoding {
      Encoding::Utf16 => {
        // FIXME: the single-byte argument bug is right here. must parse control sequences here
        let mut buf = [0; 2];
        self.raw_strings = self.strings.iter()
          .map(|s| {
            s.encode_utf16()
              .flat_map(|u| {
                self.msbt().header.endianness.write_u16(&mut buf[..], u).expect("failed to write to array");
                buf.to_vec()
              })
              .collect()
          })
          .collect();
      },
      Encoding::Utf8 => self.raw_strings = self.strings.iter().map(|x| x.as_bytes().to_vec()).collect(),
    }
    self.update();
  }

  pub fn raw_strings(&self) -> &[Vec<u8>] {
    &self.raw_strings
  }

  // can't implement this even incorrectly until control sequence parsing
  // pub fn set_raw_strings<I, S>(&mut self, strings: I) -> crate::error::Result<()>
  //   where I: IntoIterator<Item = S>,
  //         S: Into<Vec<u8>>,
  // {
  //   let _raw_strings: Vec<Vec<u8>> = strings.into_iter().map(Into::into).collect();
  //   unimplemented!() // FIXME
  // }

  pub(crate) fn update(&mut self) {
    self.string_count = self.strings.len() as u32;
    let all_str_len = self.strings.iter().flat_map(|x| x.encode_utf16()).count() * 2;
    let new_size = all_str_len // length of all strings
      + self.string_count as usize * std::mem::size_of::<u32>() // all offsets
      + std::mem::size_of_val(&self.string_count); // length of string count
    self.section.size = new_size as u32;
  }
}
