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

use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;
use std::result;
use std::str;

/// Specifies a memory region within the cartridge address space.
/// Lower bound is inclusive; upper bound is exclusive.
pub struct Region(usize, usize);

#[derive(Clone, Eq, PartialEq)]
pub enum Component {
  ROM,
  MBC(MBCCount),
  Battery,
  MMM,
  RAM,
  SRAM,
  Timer,
  Rumble,
  PocketCam,
  BandaiTAMA5,
  HudsonHUC1,
  HudsonHUC3,
}

#[derive(Clone, Eq, PartialEq)]
pub enum MBCCount {
  One,
  Two,
  Three,
  Five
}

pub struct Cartridge {
  title: String,
  rom: CartROM,
  components: Vec<Component>,
}

struct CartROM {
  bytes: Vec<u8>,
}
pub type Result<T> = result::Result<T, CartErr>;

pub enum CartErr {
  UnknownComponents(u8),
}

// TODO is there a better way?
mod regions {
  use super::Region;
  pub const META_ENTRY: Region         = Region(0x100, 0x104);
  pub const META_LOGO: Region          = Region(0x104, 0x134);
  pub const META_TITLE: Region         = Region(0x134, 0x144);
  pub const META_MANUFACTURER: Region  = Region(0x13F, 0x143);
  pub const META_CGB_FLAG: Region      = Region(0x143, 0x144);
  pub const META_LICENSEE: Region      = Region(0x144, 0x146);
  pub const META_SGB: Region           = Region(0x146, 0x147);
  pub const META_COMPONENTS: Region    = Region(0x147, 0x148);
  pub const META_ROM_SIZE: Region      = Region(0x148, 0x149);
  pub const META_RAM_SIZE: Region      = Region(0x149, 0x14A);
  pub const META_DEST: Region          = Region(0x14A, 0x14B);
  pub const META_LICENSEE_OLD: Region  = Region(0x14B, 0x14C);
  pub const META_VERSION: Region       = Region(0x14C, 0x14D);
  pub const META_CHECKSUM_HDR: Region  = Region(0x14D, 0x14E);
  pub const META_CHECKSUM_ALL: Region  = Region(0x14E, 0x150);
}

impl Region {

  pub fn cut_slice<'a>(&self, vec: &'a [u8]) -> &'a [u8] {
    &vec[self.0 .. self.1 - 1]
  }

}

impl<'a> Cartridge {

  pub fn new(bytes: Vec<u8>) -> Result<Cartridge> {
    let rom = try!(CartROM::from_raw_bytes(bytes));

    let title = read_title(&rom.bytes);
    // TODO do something with the error
    let components = try!(decode_components(rom.bytes.as_slice()));

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
}

// TODO use more specific param than just byte vec
// TODO ...is there any way to determine that we're not reading garbage? does it matter?
fn read_title(rom: &[u8]) -> String {
  String::from_utf8_lossy(regions::META_TITLE.cut_slice(rom)).into_owned()
}

// TODO yield meaningful error type
// TODO use more specific param than just byte vec
fn decode_components(rom: &[u8]) -> Result<Vec<Component>> {
  let comps = match regions::META_COMPONENTS.cut_slice(rom)[0] {
    0x0 => vec![Component::ROM],
    0x1 => vec![Component::ROM, Component::MBC(MBCCount::One)],
    0x2 => vec![Component::ROM, Component::MBC(MBCCount::One), Component::RAM],
    0x3 => vec![Component::ROM, Component::MBC(MBCCount::One), Component::RAM,
                Component::Battery],
    0x5 => vec![Component::ROM, Component::MBC(MBCCount::Two)],
    0x6 => vec![Component::ROM, Component::MBC(MBCCount::Two), Component::Battery],
    0x8 => vec![Component::ROM, Component::RAM],
    0xB => vec![Component::ROM, Component::MMM],
    0xC => vec![Component::ROM, Component::MMM, Component::SRAM],
    0xD => vec![Component::ROM, Component::MMM, Component::SRAM,
                  Component::Battery],
    0xF => vec![Component::ROM, Component::MBC(MBCCount::Three), Component::Timer,
                  Component::Battery],
    0x10 => vec![Component::ROM, Component::MBC(MBCCount::Three), Component::Timer,
                  Component::RAM, Component::Battery],
    0x11 => vec![Component::ROM, Component::MBC(MBCCount::Three)],
    0x12 => vec![Component::ROM, Component::MBC(MBCCount::Three), Component::RAM],
    0x13 => vec![Component::ROM, Component::MBC(MBCCount::Five), Component::RAM,
                  Component::Battery],
    0x19 => vec![Component::ROM, Component::MBC(MBCCount::Five)],
    0x1A => vec![Component::ROM, Component::MBC(MBCCount::Five), Component::RAM],
    0x1B => vec![Component::ROM, Component::MBC(MBCCount::Five), Component::RAM,
                  Component::Battery],
    0x1C => vec![Component::ROM, Component::MBC(MBCCount::Five), Component::Rumble],
    0x1D => vec![Component::ROM, Component::MBC(MBCCount::Five), Component::Rumble,
                  Component::SRAM],
    0x1E => vec![Component::ROM, Component::MBC(MBCCount::Five), Component::Rumble,
                  Component::SRAM, Component::Battery],
    0x1F => vec![Component::PocketCam],
    0xFD => vec![Component::BandaiTAMA5],
    0xFE => vec![Component::HudsonHUC3],
    0xFF => vec![Component::HudsonHUC1],
    x => return Err(CartErr::UnknownComponents(x)),
  };

  Ok(comps)
}

fn decode_is_cgb() {

}
