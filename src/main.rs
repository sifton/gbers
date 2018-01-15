#![feature(try_from)]

mod hw;

fn main() {
  let c = hw::cart::Cartridge::from_file("pky.gbc").ok().unwrap().ok().unwrap();
  println!("{}", c.title());
}
