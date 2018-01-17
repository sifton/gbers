#![feature(try_from)]

mod hw;

fn main() {
  let c = hw::cart::Cartridge::from_file("pky.gbc");

  match c {
    Ok(y) => {
      println!("Title: {}", y.title());
      println!("COMPONENTS LIST:");
      for comp in y.components() {
        println!("  {:?}", comp);
      }
      println!("Is CGB: {}", y.is_cgb())
    },
    Err(y) => println!("{:?}", y),
  }

}
