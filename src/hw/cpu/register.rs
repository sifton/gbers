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
