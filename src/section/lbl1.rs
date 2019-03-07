use crate::Msbt;
use super::Section;

use std::ptr::NonNull;

#[derive(Debug)]
pub struct Lbl1 {
  pub(crate) msbt: NonNull<Msbt>,
  pub(crate) section: Section,
  pub(crate) group_count: u32,
  pub(crate) groups: Vec<Group>,
  pub(crate) labels: Vec<Label>,
}

impl Lbl1 {
  pub fn msbt(&self) -> &Msbt {
    unsafe { self.msbt.as_ref() }
  }

  pub fn section(&self) -> &Section {
    &self.section
  }

  pub fn group_count(&self) -> u32 {
    self.group_count
  }

  pub fn groups(&self) -> &[Group] {
    &self.groups
  }

  pub fn labels(&self) -> &[Label] {
    &self.labels
  }
}

#[derive(Debug)]
pub struct Group {
  pub(crate) label_count: u32,
  pub(crate) offset: u32,
}

impl Group {
  pub fn label_count(&self) -> u32 {
    self.label_count
  }

  pub fn offset(&self) -> u32 {
    self.offset
  }
}

#[derive(Debug)]
pub struct Label {
  pub(crate) name: String,
  pub(crate) index: u32,
  pub(crate) checksum: u32,
  pub(crate) value: String,
  pub(crate) value_raw: Vec<u8>,
}

impl Label {
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn index(&self) -> u32 {
    self.index
  }

  pub fn checksum(&self) -> u32 {
    self.checksum
  }

  pub fn value(&self) -> &str {
    &self.value
  }

  pub fn value_raw(&self) -> &[u8] {
    &self.value_raw
  }
}
