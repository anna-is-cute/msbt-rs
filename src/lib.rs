use byteordered::{Endianness, Endian};

use std::{
  collections::BTreeMap,
  io::{Read, Seek, SeekFrom, Write},
};

mod counter;
mod error;

use self::{
  counter::Counter,
  error::{Error, Result},
};

const HEADER_MAGIC: [u8; 8] = *b"MsgStdBn";
// const LABEL_HASH_MAGIC: u16 = 0x492;
// const LABEL_MAX_LEN: u8 = 64;
// const BYTE_ORDER_OFFSET: u8 = 0x8;
// const HEADER_SIZE: u8 = 0x20;
const PADDING_CHAR: u8 = 0xAB;
const PADDING_LENGTH: usize = 16;

#[derive(Debug)]
pub enum SectionTag {
  Lbl1,
  Nli1,
  Ato1,
  Atr1,
  Tsy1,
  Txt2,
}

#[derive(Debug)]
pub struct Msbt {
  header: Header,
  section_order: Vec<SectionTag>,
  lbl1: Option<Lbl1>,
  nli1: Option<Nli1>,
  ato1: Option<Ato1>,
  atr1: Option<Atr1>,
  tsy1: Option<Tsy1>,
  txt2: Option<Txt2>,
}

impl Msbt {
  pub fn from_reader<R: Read + Seek>(reader: R) -> Result<Self> {
    MsbtReader::new(reader).map(MsbtReader::into_msbt)
  }

  pub fn write_to<W: Write>(&self, writer: W) -> Result<()> {
    let mut writer = MsbtWriter::new(self, writer);
    writer.write_header()?;
    for tag in &self.section_order {
      match *tag {
        SectionTag::Lbl1 => writer.write_lbl1()?,
        SectionTag::Nli1 => writer.write_nli1()?,
        SectionTag::Ato1 => writer.write_ato1()?,
        SectionTag::Atr1 => writer.write_atr1()?,
        SectionTag::Tsy1 => writer.write_tsy1()?,
        SectionTag::Txt2 => writer.write_txt2()?,
      }
    }
    Ok(())
  }
}

#[derive(Debug)]
pub struct MsbtWriter<'a, W> {
  writer: Counter<W>,
  msbt: &'a Msbt,
}

impl<'a, W: Write> MsbtWriter<'a, W> {
  fn new(msbt: &'a Msbt, writer: W) -> Self {
    MsbtWriter {
      msbt,
      writer: Counter::new(writer),
    }
  }

  fn write_header(&mut self) -> Result<()> {
    self.writer.write_all(&self.msbt.header.magic).map_err(Error::Io)?;
    let endianness = match self.msbt.header.endianness {
      Endianness::Big => [0xFE, 0xFF],
      Endianness::Little => [0xFF, 0xFE],
    };
    self.writer.write_all(&endianness).map_err(Error::Io)?;
    self.msbt.header.endianness.write_u16(&mut self.writer, self.msbt.header._unknown_1).map_err(Error::Io)?;
    let encoding_byte = match self.msbt.header.encoding {
      Encoding::Utf8 => 0x00,
      Encoding::Utf16 => 0x01,
    };
    self.writer.write_all(&[encoding_byte, self.msbt.header._unknown_2]).map_err(Error::Io)?;
    self.msbt.header.endianness.write_u16(&mut self.writer, self.msbt.header.len).map_err(Error::Io)?;
    self.msbt.header.endianness.write_u16(&mut self.writer, self.msbt.header._unknown_3).map_err(Error::Io)?;
    self.msbt.header.endianness.write_u32(&mut self.writer, self.msbt.header.file_size).map_err(Error::Io)?;
    self.writer.write_all(&self.msbt.header.padding).map_err(Error::Io)
  }

  fn write_section(&mut self, section: &Section) -> Result<()> {
    self.writer.write_all(&section.magic).map_err(Error::Io)?;
    self.msbt.header.endianness.write_u32(&mut self.writer, section.size).map_err(Error::Io)?;
    self.writer.write_all(&section.padding).map_err(Error::Io)
  }

  fn write_group(&mut self, group: &Group) -> Result<()> {
    self.msbt.header.endianness.write_u32(&mut self.writer, group.label_count).map_err(Error::Io)?;
    self.msbt.header.endianness.write_u32(&mut self.writer, group.offset).map_err(Error::Io)
  }

  fn write_lbl1(&mut self) -> Result<()> {
    if let Some(ref lbl1) = self.msbt.lbl1 {
      self.write_section(&lbl1.section)?;
      self.msbt.header.endianness.write_u32(&mut self.writer, lbl1.group_count).map_err(Error::Io)?;
      for group in &lbl1.groups {
        self.write_group(group)?;
      }
      for label in &lbl1.labels {
        self.writer.write_all(&[label.name.len() as u8]).map_err(Error::Io)?;
        self.writer.write_all(label.name.as_bytes()).map_err(Error::Io)?;
        self.msbt.header.endianness.write_u32(&mut self.writer, label.index).map_err(Error::Io)?;
      }

      self.write_padding()?;
    }
    Ok(())
  }

  pub fn write_nli1(&mut self) -> Result<()> {
    if let Some(ref nli1) = self.msbt.nli1 {
      self.write_section(&nli1.section)?;

      if nli1.section.size > 0 {
        self.msbt.header.endianness.write_u32(&mut self.writer, nli1.id_count).map_err(Error::Io)?;

        for (&key, &val) in &nli1.global_ids {
          self.msbt.header.endianness.write_u32(&mut self.writer, val).map_err(Error::Io)?;
          self.msbt.header.endianness.write_u32(&mut self.writer, key).map_err(Error::Io)?;
        }
      }

      self.write_padding()?;
    }

    Ok(())
  }

  pub fn write_txt2(&mut self) -> Result<()> {
    if let Some(ref txt2) = self.msbt.txt2 {
      let strings: Vec<Vec<u16>> = txt2.strings
        .iter()
        .map(|x| x.encode_utf16().collect::<Vec<_>>())
        .collect();

      self.write_section(&txt2.section)?;

      // write string count
      self.msbt.header.endianness.write_u32(&mut self.writer, txt2.string_count).map_err(Error::Io)?;

      // write offsets
      let mut total = 0;
      for s in &strings {
        let offset = txt2.string_count * 4 + 4 + total;
        total += s.len() as u32 * 2;
        self.msbt.header.endianness.write_u32(&mut self.writer, offset).map_err(Error::Io)?;
      }

      // write strings
      for s in &strings {
        for &utf16_byte in s {
          self.msbt.header.endianness.write_u16(&mut self.writer, utf16_byte).map_err(Error::Io)?;
        }
      }

      self.write_padding()?;
    }

    Ok(())
  }

  pub fn write_ato1(&mut self) -> Result<()> {
    if let Some(ref ato1) = self.msbt.ato1 {
      self.write_section(&ato1.section)?;
      self.writer.write_all(&ato1._unknown).map_err(Error::Io)?;

      self.write_padding()?;
    }

    Ok(())
  }

  pub fn write_atr1(&mut self) -> Result<()> {
    if let Some(ref atr1) = self.msbt.atr1 {
      self.write_section(&atr1.section)?;
      self.writer.write_all(&atr1._unknown).map_err(Error::Io)?;

      self.write_padding()?;
    }

    Ok(())
  }

  pub fn write_tsy1(&mut self) -> Result<()> {
    if let Some(ref tsy1) = self.msbt.tsy1 {
      self.write_section(&tsy1.section)?;
      self.writer.write_all(&tsy1._unknown).map_err(Error::Io)?;

      self.write_padding()?;
    }

    Ok(())
  }

  fn write_padding(&mut self) -> Result<()> {
    let remainder = self.writer.written() % PADDING_LENGTH;
    if remainder == 0 {
      return Ok(());
    }

    self.writer.write_all(&vec![PADDING_CHAR; PADDING_LENGTH - remainder]).map_err(Error::Io)
  }
}

#[derive(Debug)]
pub struct MsbtReader<R> {
  reader: R,
  section_order: Vec<SectionTag>,
  header: Header,
  lbl1: Option<Lbl1>,
  nli1: Option<Nli1>,
  ato1: Option<Ato1>,
  atr1: Option<Atr1>,
  tsy1: Option<Tsy1>,
  txt2: Option<Txt2>,
}

impl<R: Read + Seek> MsbtReader<R> {
  fn new(mut reader: R) -> Result<Self> {
    let header = Header::from_reader(&mut reader)?;

    let mut msbt = MsbtReader {
      reader,
      header,
      lbl1: None,
      nli1: None,
      ato1: None,
      atr1: None,
      tsy1: None,
      txt2: None,
      section_order: Vec::with_capacity(6),
    };

    msbt.read_sections()?;

    Ok(msbt)
  }

  fn into_msbt(self) -> Msbt {
    Msbt {
      header: self.header,
      section_order: self.section_order,
      lbl1: self.lbl1,
      nli1: self.nli1,
      ato1: self.ato1,
      atr1: self.atr1,
      tsy1: self.tsy1,
      txt2: self.txt2,
    }
  }

  fn skip_padding(&mut self) -> Result<()> {
    let mut buf = [0; 16];
    loop {
      let read = self.reader.read(&mut buf).map_err(Error::Io)?;
      if read == 0 {
        return Ok(());
      }
      if let Some(i) = buf[..read].iter().position(|&x| x != PADDING_CHAR) {
        self.reader.seek(SeekFrom::Current(i as i64 - 16)).map_err(Error::Io)?;
        return Ok(());
      }
    }
  }

  pub fn read_sections(&mut self) -> Result<()> {
    let mut peek = [0; 4];
    loop {
      match self.reader.read_exact(&mut peek) {
        Ok(()) => {},
        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
        Err(e) => return Err(Error::Io(e)),
      }

      self.reader.seek(SeekFrom::Current(-4)).map_err(Error::Io)?;

      match &peek {
        b"LBL1" => {
          self.lbl1 = Some(self.read_lbl1()?);
          self.section_order.push(SectionTag::Lbl1);
        },
        b"ATR1" => {
          self.atr1 = Some(self.read_atr1()?);
          self.section_order.push(SectionTag::Atr1);
        },
        b"ATO1" => {
          self.ato1 = Some(self.read_ato1()?);
          self.section_order.push(SectionTag::Ato1);
        },
        b"TSY1" => {
          self.tsy1 = Some(self.read_tsy1()?);
          self.section_order.push(SectionTag::Tsy1);
        },
        b"TXT2" => {
          self.txt2 = Some(self.read_txt2()?);
          self.section_order.push(SectionTag::Txt2);
        },
        b"NLI1" => {
          self.nli1 = Some(self.read_nli1()?);
          self.section_order.push(SectionTag::Nli1);
        },
        _ => return Err(Error::InvalidSection(peek)),
      }

      self.skip_padding()?;
    }
  }

  pub fn read_lbl1(&mut self) -> Result<Lbl1> {
    let section = self.read_section()?;

    if &section.magic != b"LBL1" {
      return Err(Error::InvalidMagic);
    }

    let group_count = self.header.endianness.read_u32(&mut self.reader).map_err(Error::Io)?;

    let mut groups = Vec::with_capacity(group_count as usize);

    for _ in 0..group_count {
      groups.push(self.read_group()?);
    }

    let mut labels = Vec::with_capacity(groups.iter().map(|x| x.label_count as usize).sum());

    let mut buf = [0; 1];
    for (i, group) in groups.iter().enumerate() {
      for _ in 0..group.label_count {
        self.reader.read_exact(&mut buf).map_err(Error::Io)?;
        let str_len = buf[0] as usize;

        let mut str_buf = vec![0; str_len];
        self.reader.read_exact(&mut str_buf).map_err(Error::Io)?;
        let name = String::from_utf8(str_buf).map_err(Error::InvalidUtf8)?;
        let index = self.header.endianness.read_u32(&mut self.reader).map_err(Error::Io)?;
        let checksum = i as u32;

        labels.push(Label {
          name,
          index,
          checksum,
          value: Default::default(), // value not parsed until later
        });
      }
    }

    Ok(Lbl1 {
      section,
      group_count,
      groups,
      labels,
    })
  }

  pub fn read_atr1(&mut self) -> Result<Atr1> {
    let section = self.read_section()?;
    let mut unknown = vec![0; section.size as usize];
    self.reader.read_exact(&mut unknown).map_err(Error::Io)?;

    Ok(Atr1 {
      section,
      _unknown: unknown,
    })
  }

  pub fn read_ato1(&mut self) -> Result<Ato1> {
    let section = self.read_section()?;
    let mut unknown = vec![0; section.size as usize];
    self.reader.read_exact(&mut unknown).map_err(Error::Io)?;

    Ok(Ato1 {
      section,
      _unknown: unknown,
    })
  }

  pub fn read_tsy1(&mut self) -> Result<Tsy1> {
    let section = self.read_section()?;
    let mut unknown = vec![0; section.size as usize];
    self.reader.read_exact(&mut unknown).map_err(Error::Io)?;

    Ok(Tsy1 {
      section,
      _unknown: unknown,
    })
  }

  pub fn read_txt2(&mut self) -> Result<Txt2> {
    let section = self.read_section()?;
    let string_count = self.header.endianness.read_u32(&mut self.reader).map_err(Error::Io)? as usize;

    let mut offsets = Vec::with_capacity(string_count);
    let mut strings = Vec::with_capacity(string_count);

    for _ in 0..string_count {
      offsets.push(self.header.endianness.read_u32(&mut self.reader).map_err(Error::Io)?);
    }

    for i in 0..string_count {
      let next_str_end = if i == string_count - 1 {
        section.size
      } else {
        offsets[i + 1]
      };
      let str_len = next_str_end - offsets[i];
      let mut str_buf = vec![0; str_len as usize];
      self.reader.read_exact(&mut str_buf).map_err(Error::Io)?;
      let value = match self.header.encoding {
        Encoding::Utf8 => String::from_utf8(str_buf).map_err(Error::InvalidUtf8)?,
        Encoding::Utf16 => {
          let u16s = (0..str_buf.len() / 2)
            .map(|i| self.header.endianness.read_u16(&str_buf[i * 2..]).map_err(Error::Io))
            .collect::<Result<Vec<u16>>>()?;
          String::from_utf16(&u16s).map_err(Error::InvalidUtf16)?
        },
      };

      strings.push(value);
    }

    if let Some(ref mut lbl1) = self.lbl1 {
      for label in &mut lbl1.labels {
        label.value = strings[label.index as usize].clone();
      }
    }

    Ok(Txt2 {
      section,
      string_count: string_count as u32,
      strings,
    })
  }

  pub fn read_nli1(&mut self) -> Result<Nli1> {
    let section = self.read_section()?;

    let mut map = BTreeMap::default();
    let mut id_count = 0;

    if section.size > 0 {
      id_count = self.header.endianness.read_u32(&mut self.reader).map_err(Error::Io)?;

      for _ in 0..id_count {
        let val = self.header.endianness.read_u32(&mut self.reader).map_err(Error::Io)?;
        let key = self.header.endianness.read_u32(&mut self.reader).map_err(Error::Io)?;
        map.insert(key, val);
      }
    }

    Ok(Nli1 {
      section,
      id_count,
      global_ids: map,
    })
  }

  pub fn read_group(&mut self) -> Result<Group> {
    let label_count = self.header.endianness.read_u32(&mut self.reader).map_err(Error::Io)?;
    let offset = self.header.endianness.read_u32(&mut self.reader).map_err(Error::Io)?;

    Ok(Group {
      label_count,
      offset,
    })
  }

  pub fn read_section(&mut self) -> Result<Section> {
    let mut magic = [0; 4];
    let mut padding = [0; 8];

    self.reader.read_exact(&mut magic).map_err(Error::Io)?;
    let size = self.header.endianness.read_u32(&mut self.reader).map_err(Error::Io)?;
    self.reader.read_exact(&mut padding).map_err(Error::Io)?;

    Ok(Section {
      magic,
      size,
      padding,
    })
  }
}

#[derive(Debug)]
pub struct Header {
  magic: [u8; 8],
  endianness: Endianness,
  _unknown_1: u16,
  encoding: Encoding,
  _unknown_2: u8,
  len: u16,
  _unknown_3: u16,
  file_size: u32,
  padding: [u8; 10],
}

impl Header {
  pub fn from_reader(mut reader: &mut Read) -> Result<Self> {
    let mut buf = [0u8; 10];
    reader.read_exact(&mut buf[..8]).map_err(Error::Io)?;

    let mut magic = [0u8; 8];
    magic.swap_with_slice(&mut buf[..8]);
    if magic != HEADER_MAGIC {
      return Err(Error::InvalidMagic);
    }

    reader.read_exact(&mut buf[..2]).map_err(Error::Io)?;

    let endianness = if buf[..2] == [0xFE, 0xFF] {
      Endianness::Big
    } else if buf[..2] == [0xFF, 0xFE] {
      Endianness::Little
    } else {
      return Err(Error::InvalidBom);
    };

    let unknown_1 = endianness.read_u16(&mut reader).map_err(Error::Io)?;

    reader.read_exact(&mut buf[..1]).map_err(Error::Io)?;
    let encoding = match buf[0] {
      0x00 => Encoding::Utf8,
      0x01 => Encoding::Utf16,
      x => return Err(Error::InvalidEncoding(x)),
    };

    reader.read_exact(&mut buf[..1]).map_err(Error::Io)?;
    let unknown_2 = buf[0];

    let len = endianness.read_u16(&mut reader).map_err(Error::Io)?;

    let unknown_3 = endianness.read_u16(&mut reader).map_err(Error::Io)?;

    let file_size = endianness.read_u32(&mut reader).map_err(Error::Io)?;

    reader.read_exact(&mut buf[..10]).map_err(Error::Io)?;
    let padding = buf;

    Ok(Header {
      magic,
      endianness,
      encoding,
      len,
      file_size,
      padding,
      _unknown_1: unknown_1,
      _unknown_2: unknown_2,
      _unknown_3: unknown_3,
    })
  }
}

#[derive(Debug)]
#[repr(u8)]
pub enum Encoding {
  Utf8 = 0x00,
  Utf16 = 0x01,
}

#[derive(Debug)]
pub struct Section {
  magic: [u8; 4],
  size: u32,
  padding: [u8; 8],
}

#[derive(Debug)]
pub struct Lbl1 {
  section: Section,
  group_count: u32,
  groups: Vec<Group>,
  labels: Vec<Label>,
}

#[derive(Debug)]
pub struct Group {
  label_count: u32,
  offset: u32,
}

#[derive(Debug)]
pub struct Label {
  name: String,
  index: u32,
  checksum: u32,
  value: String,
}

#[derive(Debug)]
pub struct Nli1 {
  section: Section,
  id_count: u32,

  global_ids: BTreeMap<u32, u32>,
}

#[derive(Debug)]
pub struct Ato1 {
  section: Section,
  _unknown: Vec<u8>, // large collection of 0xFF
}

#[derive(Debug)]
pub struct Atr1 {
  section: Section,
  _unknown: Vec<u8>, // tons of unknown data
}

#[derive(Debug)]
pub struct Tsy1 {
  section: Section,
  _unknown: Vec<u8>, // tons of unknown data
}

#[derive(Debug)]
pub struct Txt2 {
  section: Section,
  string_count: u32,

  strings: Vec<String>,
}
