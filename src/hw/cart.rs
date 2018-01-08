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

use std::any::TypeId;
use std::convert::{Into, TryInto};
use std::fs;
use std::io;
use std::io::Read;
use std::marker::PhantomData;
use std::mem;
use std::path::Path;
use std::result;
use std::str;

/// Specifies a memory region within the cartridge address space.
/// Lower bound is inclusive; upper bound is exclusive.
pub struct Region<'a, T: 'a>(usize, usize, PhantomData<&'a T>);

#[derive(Clone, Eq, PartialEq)]
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

pub struct Cartridge {
  title: String,
  rom: CartROM,
  components: Vec<Component>,
}

struct CartROM {
  bytes: Vec<u8>,
}

struct CartROMSlice<'a, T: PartialEq + 'static> {
  rom: &'a CartROM,
  region: &'a Region<'a, T>,
  bytes: &'a T,
}

#[derive(Clone, Eq, PartialEq)]
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

#[derive(Clone, PartialEq, Eq)]
pub enum RAMNum {
  N0,
  N1_2kB,
  N1_8kB,
  N3,
  N4
}

#[derive (Clone, PartialEq, Eq)]
pub enum MBCNum {
  N1,
  N2,
  N3,
  N5
}

pub type Result<T> = result::Result<T, CartErr>;

pub enum CartErr {
  UnknownComponents(u8),
  UnknownROMSize(usize),
  UnknownRAMSize(usize),
  RegionOOB,
}

const KILOBYTE_BYTES: usize = 1024;

// TODO is there a better way?
mod regions {
  use std::marker::PhantomData;
  use super::Region;
  pub const META_ENTRY: Region<[u8; 0x4]>  = Region(0x100, 0x104, PhantomData);
  pub const META_LOGO: Region<[u8; 0x30]>   = Region(0x104, 0x134, PhantomData);
  pub const META_TITLE: Region<[u8; 0x10]>  = Region(0x134, 0x144, PhantomData);
  pub const META_MANUFACTURER: Region<u32>  = Region(0x13F, 0x143, PhantomData);
  pub const META_CGB_FLAG: Region<u8>      = Region(0x143, 0x144, PhantomData);
  pub const META_LICENSEE: Region<u16>      = Region(0x144, 0x146, PhantomData);
  pub const META_SGB: Region<u8>           = Region(0x146, 0x147, PhantomData);
  pub const META_COMPONENTS: Region<u8>    = Region(0x147, 0x148, PhantomData);
  pub const META_ROM_SIZE: Region<u8>      = Region(0x148, 0x149, PhantomData);
  pub const META_RAM_SIZE: Region<u8>      = Region(0x149, 0x14A, PhantomData);
  pub const META_DEST: Region<u8>          = Region(0x14A, 0x14B, PhantomData);
  pub const META_LICENSEE_OLD: Region<u8>  = Region(0x14B, 0x14C, PhantomData);
  pub const META_VERSION: Region<u8>       = Region(0x14C, 0x14D, PhantomData);
  pub const META_CHECKSUM_HDR: Region<u8>  = Region(0x14D, 0x14E, PhantomData);
  pub const META_CHECKSUM_ALL: Region<u16>  = Region(0x14E, 0x150, PhantomData);
}

impl<'a, T> Region<'a, T> where T: PartialEq {

  fn is_in_bounds(&self, rom: &'a CartROM) -> bool {
    !(self.0 < 0 || self.0 >= rom.size_bytes()
      || self.1 < 0 || self.1 < self.0 || self.1 >= rom.size_bytes())
  }

}

impl<'a> Cartridge {

  pub fn new(bytes: Vec<u8>) -> Result<Cartridge> {
    let rom = try!(CartROM::from_raw_bytes(bytes));

    let title = try!(read_title(&rom));
    let components = try!(decode_components(&rom));

    let rom = Cartridge {
      title: title,
      rom: rom,
      components: components,
    };

    Ok(rom)
  }

  // TODO condense into one Result<_, _>
  pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Result<Cartridge>> {
    let rom: Vec<u8> = {
      let mut file = try!(fs::File::open(path));
      let mut bytes = Vec::<u8>::new();
      try!(file.read_to_end(&mut bytes));
      bytes
    };

    Ok(Cartridge::new(rom))
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


}

impl CartROM {
  fn from_raw_bytes(bytes: Vec<u8>) -> Result<CartROM> {
    Ok(CartROM {
      bytes,
    })
  }

  fn region<T>(&self, region: &'static Region<T>) -> Result<CartROMSlice<T>> where T: PartialEq{
    CartROMSlice::try_new(self, region)
  }

  fn size_bytes(&self) -> usize {
    self.bytes.len()
  }
}

impl<'a, T> CartROMSlice<'a, T> where T: PartialEq {
  fn try_new(rom: &'a CartROM, region: &'static Region<T>) -> Result<CartROMSlice<'a, T>> where T: PartialEq {
    if region.is_in_bounds(rom)
    {
      let fixed_size: &T = unsafe { mem::transmute_copy(&rom.bytes[region.0]) };
      return Ok(CartROMSlice {
        rom,
        region,
        bytes: &fixed_size,
      })
    }
    Err(CartErr::RegionOOB)
  }

  fn bytes(&self) -> &'a T {
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
      ROMNum::N2 => 2,
      ROMNum::N4 => 4,
      ROMNum::N8 => 8,
      ROMNum::N16 => 16,
      ROMNum::N32 => 32,
      ROMNum::N64 => 64,
      ROMNum::N128 => 128,
      ROMNum::N72 => 72,
      ROMNum::N80 => 80,
      ROMNum::N96 => 96
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

// TODO use more specific param than just byte vec
// TODO ...is there any way to determine that we're not reading garbage? does it matter?
fn read_title(rom: &CartROM) -> Result<String> {
  Ok(String::from_utf8_lossy(rom.region(&regions::META_TITLE)?.bytes()).into_owned())
}

// TODO yield meaningful error type
// TODO use more specific param than just byte vec
fn decode_components(rom: &CartROM) -> Result<Vec<Component>> {
  let _romnum = try!(decode_rom_size(rom));
  let _ramnum = try!(decode_ram_size(rom));

  let comps = match *rom.region(&regions::META_COMPONENTS)?.bytes() {
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

fn decode_rom_size(rom: &CartROM) -> Result<ROMNum> {
  let raw = rom.region(&regions::META_ROM_SIZE)?.bytes();
unimplemented!()
}

fn decode_ram_size(rom: &CartROM) -> Result<RAMNum> {
unimplemented!()
}

fn decode_is_cgb() {

}
