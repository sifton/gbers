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
/*

giant enum for all instructions or one for each type?

want to be able to decode instructions correctly so that deeper inspection
is possible down the line.

type safety where possible. use types to encode information about the instructions

e.g. Instr::LD_RR(r1, r2)
*/
use std::convert::{Into, TryFrom, TryInto};
use std::result;

#[derive(PartialEq)]
enum Prefix {
  CB = 0xCB,
  DD = 0xDD,
  ED = 0xED,
  FD = 0xFD
}

const PREFIX_CB: u8 = Prefix::CB as u8;
const PREFIX_DD: u8 = Prefix::DD as u8;
const PREFIX_ED: u8 = Prefix::ED as u8;
const PREFIX_FD: u8 = Prefix::FD as u8;

enum Immediate {
  Zero,
  One(u8),
  Two(u16)
}

enum Opcode {

}

enum Instr {
  Single {
    prefix: Option<Prefix>,
    opcode: Opcode,
    displace: Option<i8>,
    immed: Option<Immediate>
  },
  SpecialDD {
    displace: i8,
    opcode: Opcode
  },
  SpecialFD {
    displace: i8,
    opcode: Opcode
  }
}

type Result<T> = result::Result<T, decode::DecodeErr>;

impl Instr {

  pub fn decode(raw: &[u8]) -> Result<Instr> {
    // inspect the first byte
    unimplemented!()
  }

}

impl Into<u8> for Prefix {
  fn into(self) -> u8 {
    match self {
      Prefix::CB => PREFIX_CB,
      Prefix::DD => PREFIX_DD,
      Prefix::ED => PREFIX_ED,
      Prefix::FD => PREFIX_FD
    }
  }
}

impl TryFrom<u8> for Prefix {
  type Error = decode::DecodeErr;
  fn try_from(raw: u8) -> result::Result<Self, Self::Error> {
    match raw {
      PREFIX_CB => Ok(Prefix::CB),
      PREFIX_DD => Ok(Prefix::DD),
      PREFIX_ED => Ok(Prefix::ED),
      PREFIX_FD => Ok(Prefix::FD),
      _ => Err(decode::DecodeErr::UnknownPrefix(raw))
    }
  }
}

mod decode {
  use std::result;

  pub type Result<T> = result::Result<T, DecodeErr>;

  /// Marker trait for types eligible to be used as Decoder states.
  pub trait DecoderState {}

  pub struct Decoder<S: DecoderState> {
    bytes: [u8; 4],
    state: S,
  }

  pub enum DecodeErr {
    UnknownPrefix(u8),
  }

  struct Start {

  }

  struct Prefix {

  }

  struct DblPrefix {

  }

  struct Opcode {

  }

  struct Displace {

  }

  struct Immed {

  }

  impl Decoder<Start> {
    pub fn new(bytes: [u8; 4]) -> Decoder<Start> {
      Decoder {
        bytes,
        state: Start {},
      }
    }
  }

  impl From<Decoder<Start>> for Decoder<Prefix> {
    fn from(dec: Decoder<Start>) -> Decoder<Prefix> {
      unimplemented!()
    }
  }

  impl DecoderState for Start {}
  impl DecoderState for Prefix {}
  impl DecoderState for DblPrefix {}
  impl DecoderState for Opcode {}
  impl DecoderState for Displace {}
  impl DecoderState for Immed {}
}
