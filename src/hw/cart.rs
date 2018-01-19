// Copyright (c) 2018 Brett Russell
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software
// and associated documentation files (the "Software"), to deal in the Software without
// restriction, including without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
// BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use std::convert::{Into, TryFrom, TryInto};
use std::fs;
use std::io;
use std::io::Read;
use std::marker::PhantomData;
use std::mem;
use std::path::Path;
use std::result;
use std::str;

use self::regions::Region;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Component {
  ROM(ROMNum),
  MBC(MBCNum),
  Battery,
  MMM,
  RAM(RAMNum),
  SRAM,
  Timer,
  Rumble,
  PocketCam,
  BandaiTAMA5,
  HudsonHUC1,
  HudsonHUC3,
}

#[derive(Debug)]
pub struct Cartridge {
  title: String,
  is_cgb: bool,
  is_sgb: bool,
  rom: ROM,
  components: Vec<Component>,
}

#[derive(Debug)]
struct ROM {
  bytes: Vec<u8>,
}

#[derive(Debug)]
struct ROMSlice<'a, T: PartialEq + 'static> {
  rom: &'a ROM,
  region: &'a Region<'a, T>,
  bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ROMNum {
  N2,
  N4,
  N8,
  N16,
  N32,
  N64,
  N128,
  N72,
  N80,
  N96
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RAMNum {
  N0,
  N1_2kB,
  N1_8kB,
  N3,
  N4
}

#[derive (Clone, Debug, PartialEq, Eq)]
pub enum MBCNum {
  N1,
  N2,
  N3,
  N5
}

pub type Result<T> = result::Result<T, CartErr>;

#[derive(Debug)]
pub enum CartErr {
  UnknownComponents(u8),
  UnknownROMSize(usize),
  UnknownRAMSize(usize),
  IOError(io::Error),
  BadHeaderChecksum(u8, u8),
  RegionOOB,
}


const KILOBYTE_BYTES: usize = 1024;

// TODO is there a better way?
pub mod regions {
  use std::marker::PhantomData;

  /// Specifies a memory region within the cartridge address space.
  /// Lower bound is inclusive; upper bound is exclusive.
  #[derive(Debug)]
  pub struct Region<'a, T: 'a>(pub usize, pub usize, PhantomData<&'a T>);

  pub const META_ENTRY: Region<[u8; 0x4]>  = Region(0x100, 0x104, PhantomData);
  pub const META_LOGO: Region<[u8; 0x30]>   = Region(0x104, 0x134, PhantomData);
  pub const META_TITLE: Region<[u8; 0x10]>  = Region(0x134, 0x144, PhantomData);
  pub const META_MANUFACTURER: Region<u32>  = Region(0x13F, 0x143, PhantomData);
  pub const META_CGB_FLAG: Region<u8>      = Region(0x143, 0x144, PhantomData);
  pub const META_LICENSEE: Region<u16>      = Region(0x144, 0x146, PhantomData);
  pub const META_SGB_FLAG: Region<u8>           = Region(0x146, 0x147, PhantomData);
  pub const META_COMPONENTS: Region<u8>    = Region(0x147, 0x148, PhantomData);
  pub const META_ROM_SIZE: Region<u8>      = Region(0x148, 0x149, PhantomData);
  pub const META_RAM_SIZE: Region<u8>      = Region(0x149, 0x14A, PhantomData);
  pub const META_DEST: Region<u8>          = Region(0x14A, 0x14B, PhantomData);
  pub const META_LICENSEE_OLD: Region<u8>  = Region(0x14B, 0x14C, PhantomData);
  pub const META_VERSION: Region<u8>       = Region(0x14C, 0x14D, PhantomData);
  pub const META_CHECKSUM_HDR: Region<u8>  = Region(0x14D, 0x14E, PhantomData);
  pub const META_CHECKSUM_ALL: Region<u16> = Region(0x14E, 0x150, PhantomData);

  pub const RANGE_CHECKSUM: Region<[u8; 0x14D - 0x134]> = Region(0x134, 0x14D, PhantomData);

  pub const EXEC_BOOT: Region<[u8; 256]>   = Region(0x0, 0x256, PhantomData);
}

impl<'a, T> Region<'a, T> where T: PartialEq {

  fn is_in_bounds(&self, rom: &'a ROM) -> bool {
    !(self.0 >= rom.size_bytes() || self.1 < self.0
      || self.1 >= rom.size_bytes())
  }

}

impl<'a> Cartridge {

  pub fn new(bytes: Vec<u8>) -> Result<Cartridge> {
    let x = try!(Cartridge::new_no_check(bytes));

    let _ = try!(check_header_sum(&x.rom));

    Ok(x)
  }

  pub fn new_no_check(bytes: Vec<u8>) -> Result<Cartridge> {
    let rom = try!(ROM::from_raw_bytes(bytes));

    let title = try!(read_title(&rom));
    let components = try!(decode_components(&rom));
    let is_cgb = try!(decode_is_cgb(&rom));
    let is_sgb = try!(decode_is_sgb(&rom));

    let rom = Cartridge {
      title: title,
      is_cgb,
      is_sgb,
      rom: rom,
      components: components,
    };

    Ok(rom)
  }

  // TODO condense into one Result<_, _>
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Cartridge> {
    let rom: Vec<u8> = {
      let mut file = match fs::File::open(path) {
        Ok(x) => x,
        Err(x) => return Err(CartErr::IOError(x))
      };
      let mut bytes = Vec::<u8>::new();
      match file.read_to_end(&mut bytes) {
        Ok(x) => bytes,
        Err(x) => return Err(CartErr::IOError(x)),
      }
    };

    Cartridge::new(rom)
  }

  pub fn title(&'a self) -> &'a str {
    self.title.as_str()
  }

  pub fn components(&'a self) -> &'a Vec<Component> {
    &self.components
  }

  pub fn has_component(&self, cmp: Component) -> bool {
    self.components.contains(&cmp)
  }

  pub fn is_cgb(&self) -> bool {
    self.is_cgb
  }

  pub fn is_sgb(&self) -> bool {
    self.is_sgb
  }


}

impl ROM {
  fn from_raw_bytes(bytes: Vec<u8>) -> Result<ROM> {
    Ok(ROM {
      bytes,
    })
  }

  fn region<T>(&self, region: &'static Region<T>) -> Result<ROMSlice<T>> where T: PartialEq + Clone {
    ROMSlice::try_new(self, region)
  }

  fn size_bytes(&self) -> usize {
    self.bytes.len()
  }
}

impl<'a, T> ROMSlice<'a, T> where T: PartialEq + Clone {
  fn try_new(rom: &'a ROM, region: &'static Region<T>) -> Result<ROMSlice<'a, T>> where T: PartialEq {
    if region.is_in_bounds(rom)
    {
      return Ok(ROMSlice {
        rom,
        region,
        bytes: &rom.bytes[region.0 .. region.1],
      })
    }
    Err(CartErr::RegionOOB)
  }

  fn into(self) -> T {
    self.convert_from()
  }

  fn convert_from(&self) -> T {
    let converted: &T = unsafe { mem::transmute(&self.bytes[self.region.0]) };

    converted.clone()
  }

  fn bytes(&self) -> &'a [u8] {
    self.bytes
  }
}

impl Into<u8> for MBCNum {
  fn into(self) -> u8 {
    match self {
      MBCNum::N1 => 1,
      MBCNum::N2 => 2,
      MBCNum::N3 => 3,
      MBCNum::N5 => 5
    }
  }
}

impl ROMNum {
  pub fn size_bytes(self) -> usize {
    const _16KB: usize = 16 * KILOBYTE_BYTES;
    return (self as usize) * _16KB
  }
}

impl Into<usize> for ROMNum {
  fn into(self) -> usize {
    match self {
      ROMNum::N2 => 0,
      ROMNum::N4 => 1,
      ROMNum::N8 => 2,
      ROMNum::N16 => 3,
      ROMNum::N32 => 4,
      ROMNum::N64 => 5,
      ROMNum::N128 => 6,
      ROMNum::N72 => 0x52,
      ROMNum::N80 => 0x53,
      ROMNum::N96 => 0x54
    }
  }
}

impl TryFrom<usize> for ROMNum {
  type Error = CartErr;
  fn try_from(other: usize) -> Result<ROMNum> {
    match other {
      0 => Ok(ROMNum::N2),
      1 => Ok(ROMNum::N4),
      2 => Ok(ROMNum::N8),
      3 => Ok(ROMNum::N16),
      4 => Ok(ROMNum::N32),
      5 => Ok(ROMNum::N64),
      6 => Ok(ROMNum::N128),
      0x52 => Ok(ROMNum::N72),
      0x53 => Ok(ROMNum::N80),
      0x54 => Ok(ROMNum::N96),
      _ => Err(CartErr::UnknownROMSize(other))
    }
  }
}

impl RAMNum {
  pub fn size_bytes(self) -> usize {
    match self {
      RAMNum::N0 => 0,
      RAMNum::N1_2kB => 2 * KILOBYTE_BYTES,
      RAMNum::N1_8kB => 8 * KILOBYTE_BYTES,
      RAMNum::N3 => 32 * KILOBYTE_BYTES,
      RAMNum::N4 => 128 * KILOBYTE_BYTES,
    }
  }
}

impl Into<usize> for RAMNum {
  fn into(self) -> usize {
    match self {
      RAMNum::N0 => 0,
      RAMNum::N1_2kB => 1,
      RAMNum::N1_8kB => 2,
      RAMNum::N3 => 3,
      RAMNum::N4 => 4
    }
  }
}

impl TryFrom<usize> for RAMNum {
  type Error = CartErr;
  fn try_from(other: usize) -> Result<RAMNum> {
    match other {
      0 => Ok(RAMNum::N0),
      1 => Ok(RAMNum::N1_2kB),
      2 => Ok(RAMNum::N1_8kB),
      3 => Ok(RAMNum::N3),
      4 => Ok(RAMNum::N4),
      _ => Err(CartErr::UnknownRAMSize(other))
    }
  }
}


// TODO use more specific param than just byte vec
// TODO ...is there any way to determine that we're not reading garbage? does it matter?
fn read_title(rom: &ROM) -> Result<String> {
  Ok(String::from_utf8_lossy(&rom.region(&regions::META_TITLE)?.into()).into_owned())
}

fn decode_components(rom: &ROM) -> Result<Vec<Component>> {
  let _romnum = try!(decode_rom_size(rom));
  let _ramnum = try!(decode_ram_size(rom));

  let comps = match rom.region(&regions::META_COMPONENTS)?.into() {
    0x0 => vec![Component::ROM(_romnum)],
    0x1 => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N1)],
    0x2 => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N1), Component::RAM(_ramnum)],
    0x3 => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N1), Component::RAM(_ramnum),
                Component::Battery],
    0x5 => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N2)],
    0x6 => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N2), Component::Battery],
    0x8 => vec![Component::ROM(_romnum), Component::RAM(_ramnum)],
    0xB => vec![Component::ROM(_romnum), Component::MMM],
    0xC => vec![Component::ROM(_romnum), Component::MMM, Component::SRAM],
    0xD => vec![Component::ROM(_romnum), Component::MMM, Component::SRAM,
                  Component::Battery],
    0xF => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N3), Component::Timer,
                  Component::Battery],
    0x10 => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N3), Component::Timer,
                  Component::RAM(_ramnum), Component::Battery],
    0x11 => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N3)],
    0x12 => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N3), Component::RAM(_ramnum)],
    0x13 => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N5), Component::RAM(_ramnum),
                  Component::Battery],
    0x19 => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N5)],
    0x1A => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N5), Component::RAM(_ramnum)],
    0x1B => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N5), Component::RAM(_ramnum),
                  Component::Battery],
    0x1C => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N5), Component::Rumble],
    0x1D => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N5), Component::Rumble,
                  Component::SRAM],
    0x1E => vec![Component::ROM(_romnum), Component::MBC(MBCNum::N5), Component::Rumble,
                  Component::SRAM, Component::Battery],
    0x1F => vec![Component::PocketCam],
    0xFD => vec![Component::BandaiTAMA5],
    0xFE => vec![Component::HudsonHUC3],
    0xFF => vec![Component::HudsonHUC1],
    x => return Err(CartErr::UnknownComponents(x)),
  };

  Ok(comps)
}

fn decode_rom_size(rom: &ROM) -> Result<ROMNum> {
  (rom.region(&regions::META_ROM_SIZE)?.into() as usize).try_into()
}

fn decode_ram_size(rom: &ROM) -> Result<RAMNum> {
  (rom.region(&regions::META_RAM_SIZE)?.into() as usize).try_into()
}

fn decode_is_cgb(rom: &ROM) -> Result<bool> {
  let flag: u8 = rom.region(&regions::META_CGB_FLAG)?.into();
  Ok(flag == 0x80)
}

fn decode_is_sgb(rom: &ROM) -> Result<bool> {
  let flag: u8 = rom.region(&regions::META_SGB_FLAG)?.into();
  Ok(flag == 0x3)
}

fn check_header_sum(rom: &ROM) -> Result<()> {
  let bytes = rom.region(&regions::RANGE_CHECKSUM)?.into();
  let checksum = rom.region(&regions::META_CHECKSUM_HDR)?.into();

  let mut sum: isize = 0;
  for &b in bytes.into_iter() {
    sum = sum - (b as isize) - 1;
  }

  if (sum & 0xFF) as u8 == checksum {
    Ok(())
  } else {
    Err(CartErr::BadHeaderChecksum(sum as u8, checksum))
  }
}
