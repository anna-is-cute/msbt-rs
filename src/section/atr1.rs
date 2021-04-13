use crate::{
  Msbt,
  traits::{CalculatesSize, Updates},
};
use super::Section;

use std::ptr::NonNull;

#[derive(Debug)]
pub struct Atr1 {
  pub(crate) msbt: NonNull<Msbt>,
  pub(crate) section: Section,
  pub(crate) entry_count: u32,
  pub(crate) entry_size: u32,
  pub(crate) entries: Vec<Vec<u8>>,
}

impl Atr1 {
  pub fn msbt(&self) -> &Msbt {
    unsafe { self.msbt.as_ref() }
  }

  pub fn section(&self) -> &Section {
    &self.section
  }

  pub fn entry_count(&self) -> u32 {
    self.entry_count
  }
  
  pub fn entry_size(&self) -> u32 {
    self.entry_size
  }

  pub fn entries(&self) -> Vec<Vec<u8>> {
    self.entries.clone()
  }
}

impl CalculatesSize for Atr1 {
  fn calc_size(&self) -> usize {
    self.section.calc_size()
    + std::mem::size_of_val(&self.entry_count)
    + std::mem::size_of_val(&self.entry_size)
    + std::mem::size_of::<u8>() * (self.entry_count * self.entry_size) as usize
  }
}

impl Updates for Atr1 {
  fn update(&mut self) {
    self.entry_count = self.entries.len() as u32;
    self.section.size = self.calc_size() as u32 - self.section.calc_size() as u32;
  }
}
