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

impl<T> Register<T> for Reg<T> where T: Clone {
  fn get(&self) -> T {
    self.value.clone()
  }

  fn set(&mut self, new_value: T) {
    self.value = new_value;
  }
}
