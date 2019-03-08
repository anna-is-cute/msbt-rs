pub(crate) trait CalculatesSize {
  /// Calculate the size of this object when written to an MSBT.
  fn calc_size(&self) -> usize;
}

pub trait Updates {
  /// Update this object with any new changes made.
  fn update(&mut self);
}
