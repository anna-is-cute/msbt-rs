use msbt::Msbt;

use std::{
  fs::File,
  io::{BufReader, BufWriter},
};

fn main() {
  for arg in std::env::args().skip(1) {
    let f = BufReader::new(File::open(&arg).unwrap());

    let msbt = Msbt::from_reader(f).unwrap();

    let new_f = BufWriter::new(File::create(format!("{}-new", arg)).unwrap());

    msbt.write_to(new_f).unwrap();
  }
}
