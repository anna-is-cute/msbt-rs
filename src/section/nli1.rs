use crate::{
  Msbt,
  traits::CalculatesSize,
};
use super::Section;

use std::{
  collections::BTreeMap,
  ptr::NonNull,
};

#[derive(Debug)]
pub struct Nli1 {
  pub(crate) msbt: NonNull<Msbt>,
  pub(crate) section: Section,
  pub(crate) id_count: u32,
  pub(crate) global_ids: BTreeMap<u32, u32>,
}

impl Nli1 {
  pub fn msbt(&self) -> &Msbt {
    unsafe { self.msbt.as_ref() }
  }

  pub fn section(&self) -> &Section {
    &self.section
  }

  pub fn id_count(&self) -> u32 {
    self.id_count
  }

  pub fn global_ids(&self) -> &BTreeMap<u32, u32> {
    &self.global_ids
  }
}

impl CalculatesSize for Nli1 {
  fn calc_size(&self) -> usize {
    let mut base = self.section.file_size();

    if !self.global_ids.is_empty() {
      base += std::mem::size_of_val(&self.id_count)
        + std::mem::size_of::<u32>() * self.global_ids.len() * 2;
    }

    base
  }
}
