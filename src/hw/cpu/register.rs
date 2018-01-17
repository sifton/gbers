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

pub trait Register<T> {
  fn get(&self) -> T;
  fn set(&mut self, new_value: T);
}

pub trait FlagRegister: Register<u8> {
  fn is_set(&self, flag: Flag) -> bool;
}

pub struct Reg {
  value: u8,
}

pub struct CompositeReg {
  upper: Reg,
  lower: Reg,
}


pub enum Flag {
  Zero = 1 << 7,
  AddSub = 1 << 6,
  HalfCarry = 1 << 5,
  Carry = 1 << 4
}

impl Register<u16> for CompositeReg {
  fn get(&self) -> u16 {
    ((self.upper.get() << 8 + self.lower.get()) as u16)
  }

  fn set(&mut self, new_value: u16) {
    self.upper.set((new_value >> 8) as u8);
    self.lower.set((new_value & 0xFF) as u8);
  }
}

impl CompositeReg {

  pub fn upper(&self) -> &Reg {
    &self.upper
  }

  pub fn upper_mut(&mut self) -> &mut Reg {
    &mut self.upper
  }

  pub fn lower(&self) -> &Reg {
    &self.lower
  }

  pub fn lower_mut(&mut self) -> &mut Reg {
    &mut self.lower
  }
}

impl CompositeReg {
  pub fn new(initial: u16) -> Self {
    let mut x = CompositeReg {
      upper: Reg::new(0),
      lower: Reg::new(0),
    };
    x.set(initial);
    x
  }
}

impl Reg  {
  pub fn new(initial: u8) -> Self {
    Reg {
      value: initial,
    }
  }
}

impl Register<u8> for Reg {
  fn get(&self) -> u8 {
    self.value.clone()
  }

  fn set(&mut self, new_value: u8) {
    self.value = new_value;
  }
}

impl FlagRegister for Reg {
  fn is_set(&self, flag: Flag) -> bool {
    (self.get() & (flag as u8)) != 0
  }
}
