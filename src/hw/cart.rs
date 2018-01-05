// Copyright (c) 2016 Brett Russell
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

use std::collections::HashSet;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;
use std::str;

/// Specifies a memory region within the cartridge address space.
/// Lower bound is inclusive; upper bound is exclusive.
pub struct Region(usize, usize);

#[derive(Eq, PartialEq)]
pub enum Component {
  ROM,
  MBC(u8),
  Battery,
  MMM,
  RAM,
  SRAM,
  Rumble,
  PocketCam,
  BandaiTAMA5,
  HudsonHUC3,
}

pub struct Cartridge {
  title: String,
  rom: Vec<u8>,
  components: HashSet<Component>,
}

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

  pub fn cut_slice<'a>(&self, vec: &'a Vec<u8>) -> &'a [u8] {
    &vec[self.0 .. self.1 - 1]
  }

}

impl<'a> Cartridge {

  pub fn new(bytes: Vec<u8>) -> Result<Cartridge, String> {
    let title = read_title(&bytes);
    let components = read_components(&bytes);

    let rom = Cartridge {
      title: title,
      rom: bytes,
      components: components,
    };

    Ok(rom)
  }

  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Result<Cartridge, String>, io::Error> {

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

  pub fn components(&'a self) -> &'a HashSet<Component> {
    &self.components
  }

  pub fn has_component(&self, cmp: Component) -> bool {
    self.components.contains(cmp)
  }


}


fn read_title(rom: &Vec<u8>) -> String {
  String::from_utf8_lossy(regions::META_TITLE.cut_slice(rom)).into_owned()
}

fn read_components(rom: &Vec<u8>) -> Components {

  match regions::META_COMPONENTS.cut_slice(rom)[0] {
    0x0 => component::ROM,
  }

}