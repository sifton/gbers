use std::marker::PhantomData;

pub trait Register<T> {
  fn get(&self) -> T;
  fn set(&mut self, new_value: T);
}

struct Reg<T: Clone> {
  value: T,
}

struct CompositeReg<S: Clone, D: Clone> {
  upper: Reg<S>,
  lower: Reg<S>,
  data: PhantomData<D>
}

pub struct Processor {
  reg_af: CompositeReg<u8, u16>,
  reg_bc: CompositeReg<u8, u16>,
  reg_de: CompositeReg<u8, u16>,
  reg_hl: CompositeReg<u8, u16>,
  reg_sp: Reg<u16>,
  reg_pc: Reg<u16>
}

impl Register<u16> for CompositeReg<u8, u16> {
  fn get(&self) -> u16 {
    ((self.upper.get() << 8 + self.lower.get()) as u16)
  }

  fn set(&mut self, new_value: u16) {
    self.upper.set((new_value >> 8) as u8);
    self.lower.set((new_value & 0xFF) as u8);
  }
}

impl<S, D> CompositeReg<S, D> where S: Clone, D: Clone, {

  fn upper(&self) -> &Reg<S> {
    &self.upper
  }

  fn upper_mut(&mut self) -> &mut Reg<S> {
    &mut self.upper
  }

  fn lower(&self) -> &Reg<S> {
    &self.lower
  }

  fn lower_mut(&mut self) -> &mut Reg<S> {
    &mut self.lower
  }
}

impl CompositeReg<u8, u16> {
  fn new(initial: u16) -> Self {
    let mut x = CompositeReg {
      upper: Reg::new(0),
      lower: Reg::new(0),
      data: PhantomData,
    };
    x.set(initial);
    x
  }
}

impl<T> Reg<T> where T: Clone {
  fn new(initial: T) -> Self {
    Reg {
      value: initial,
    }
  }
}

impl<T> Register<T> for Reg<T> where T: Clone {
  fn get(&self) -> T {
    self.value.clone()
  }

  fn set(&mut self, new_value: T) {
    self.value = new_value;
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
