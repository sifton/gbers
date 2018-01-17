const CYCLE_INCREMENT: usize = 4;

pub struct Clock {
  freq: Frequency,
  time: usize,
}

pub enum Frequency {
  Single,
  Double
}

impl Clock {

  pub fn new(freq: Frequency) -> Clock {
    Clock::new_start_time(0, freq)
  }

  pub fn new_start_time(time: usize, freq: Frequency) -> Clock {
    Clock {
      time,
      freq,
    }
  }

  pub fn incr(&mut self) {
    self.time += CYCLE_INCREMENT;
  }

  pub fn incr_n(&mut self, n: usize) {
    self.time += CYCLE_INCREMENT * n;
  }

  pub fn time(&self) -> usize {
    self.time
  }

}
