
use super::cart;

trait Register<T> {
  fn get(&self) -> T;
  fn set(&mut self, new_value: T);
}

trait FlagRegister: Register<u8> {
  fn is_set(&self, flag: Flag) -> bool;
}

struct Reg {
  value: u8,
}

struct CompositeReg {
  upper: Reg,
  lower: Reg,
}

pub struct Processor {
  reg_af: CompositeReg,
  reg_bc: CompositeReg,
  reg_de: CompositeReg,
  reg_hl: CompositeReg,
  reg_sp: Reg,
  reg_pc: Reg
}

enum Flag {
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

  fn upper(&self) -> &Reg {
    &self.upper
  }

  fn upper_mut(&mut self) -> &mut Reg {
    &mut self.upper
  }

  fn lower(&self) -> &Reg {
    &self.lower
  }

  fn lower_mut(&mut self) -> &mut Reg {
    &mut self.lower
  }
}

impl CompositeReg {
  fn new(initial: u16) -> Self {
    let mut x = CompositeReg {
      upper: Reg::new(0),
      lower: Reg::new(0),
    };
    x.set(initial);
    x
  }
}

impl Reg where {
  fn new(initial: u8) -> Self {
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

impl Processor {
  pub fn new() -> Processor {
    Processor {
      reg_af: CompositeReg::new(0),
      reg_bc: CompositeReg::new(0),
      reg_de: CompositeReg::new(0),
      reg_hl: CompositeReg::new(0),
      reg_pc: Reg::new(0),
      reg_sp: Reg::new(0)
    }
  }

  pub fn start(&mut self) {

  }

}
