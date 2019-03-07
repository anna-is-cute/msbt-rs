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

  fn msbt_mut(&mut self) -> &mut Msbt {
    unsafe { self.msbt.as_mut() }
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

  pub fn labels_mut(&mut self) -> &mut [Label] {
    &mut self.labels
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
  pub(crate) lbl1: NonNull<Lbl1>,
  pub(crate) name: String,
  pub(crate) index: u32,
  pub(crate) checksum: u32,
}

impl Label {
  fn lbl1(&self) -> &Lbl1 {
    unsafe { self.lbl1.as_ref() }
  }

  fn lbl1_mut(&mut self) -> &mut Lbl1 {
    unsafe { self.lbl1.as_mut() }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn index(&self) -> u32 {
    self.index
  }

  pub fn checksum(&self) -> u32 {
    self.checksum
  }

  /// Gets the value of this label.
  ///
  /// Note that the value is not guaranteed to exist. The Msbt containing the Lbl1 of this label
  /// will have its Txt2 checked for this label's index, then that string returned if it exists.
  pub fn value(&self) -> Option<&str> {
    self.lbl1().msbt().txt2
      .as_ref()
      .and_then(|t| t.strings.get(self.index as usize).map(AsRef::as_ref))
  }

  /// Sets the value of this label.
  ///
  /// This checks the Txt2 of the Msbt containing the Lbl1 of this label for this label's index,
  /// then sets that index if it exists.
  pub fn set_value<S: Into<String>>(&mut self, val: S) -> Result<(), ()> {
    let string = val.into();
    let index = self.index as usize;
    let txt2 = self.lbl1_mut().msbt_mut().txt2.as_mut();
    if let Some(txt2) = txt2 {
      let txt2_str = txt2.strings.get_mut(index as usize);
      if let Some(txt2_str) = txt2_str {
        *txt2_str = string;
        txt2.update();

        return Ok(());
      }
    }

    Err(())
  }

  /// Gets the value of this label.
  ///
  /// # Panics
  ///
  /// This method will panic is the Msbt containing this label's Lbl1 does not have a Txt2 or if
  /// that Txt2 does not have a string at this label's index.
  pub unsafe fn value_unchecked(&self) -> &str {
    &self.lbl1().msbt().txt2.as_ref().unwrap().strings[self.index as usize]
  }

  pub fn value_raw(&self) -> Option<&[u8]> {
    self.lbl1().msbt().txt2
      .as_ref()
      .and_then(|t| t.raw_strings.get(self.index as usize).map(AsRef::as_ref))
  }

  pub unsafe fn value_raw_unchecked(&self) -> &[u8] {
    &self.lbl1().msbt().txt2.as_ref().unwrap().raw_strings[self.index as usize]
  }
}
