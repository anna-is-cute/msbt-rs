use crate::{
  Msbt,
  traits::CalculatesSize,
};
use super::Section;

use std::ptr::NonNull;

#[derive(Debug)]
pub struct Ato1 {
  pub(crate) msbt: NonNull<Msbt>,
  pub(crate) section: Section,
  pub(crate) _unknown: Vec<u8>, // large collection of 0xFF
}

impl Ato1 {
  pub fn msbt(&self) -> &Msbt {
    unsafe { self.msbt.as_ref() }
  }

  pub fn section(&self) -> &Section {
    &self.section
  }

  pub fn unknown_bytes(&self) -> &[u8] {
    &self._unknown
  }
}

impl CalculatesSize for Ato1 {
  fn calc_size(&self) -> usize {
    self.section.file_size() + self._unknown.len()
  }
}
