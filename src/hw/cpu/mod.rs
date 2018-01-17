
mod clock;
mod instr;
mod register;

use super::cart;

use self::register::*;

pub struct Processor {
  reg_af: CompositeReg,
  reg_bc: CompositeReg,
  reg_de: CompositeReg,
  reg_hl: CompositeReg,
  reg_sp: Reg,
  reg_pc: Reg
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
