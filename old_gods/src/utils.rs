//! Utilities
use super::time::*;

pub struct DurationMeasurement {
  start: Millis,
}


impl DurationMeasurement {
  pub fn starting_now() -> DurationMeasurement {
    DurationMeasurement {
      start: Millis::now(),
    }
  }


  pub fn millis_since_start(&self) -> u32 {
    Millis::now().millis_since(self.start.clone())
  }
}


pub fn measure<T, F: FnOnce() -> T>(f: F) -> (T, u32) {
  let m = DurationMeasurement::starting_now();
  let t = f();
  (t, m.millis_since_start())
}


pub struct FPSCounter {
  buffer: [f32; 600],
  index: usize,
  last_instant: Millis,
}


impl FPSCounter {
  pub fn new() -> FPSCounter {
    FPSCounter {
      buffer: [0.0; 600],
      index: 0,
      last_instant: Millis::now(),
    }
  }
  pub fn next_frame(&mut self) -> f32 {
    let this_instant = Millis::now();
    let delta = this_instant.millis_since(self.last_instant);
    let dt_seconds = delta as f32 / 1000.0;
    self.last_instant = this_instant;
    self.buffer[self.index] = dt_seconds;
    self.index = (self.index + 1) % self.buffer.len();
    dt_seconds
  }

  pub fn avg_frame_delta(&self) -> f32 {
    self.buffer.iter().fold(0.0, |sum, dt| sum + dt) / self.buffer.len() as f32
  }

  pub fn current_fps(&self) -> f32 {
    1.0 / self.avg_frame_delta()
  }

  /// Return the last frame's delta in seconds.
  pub fn last_delta(&self) -> f32 {
    self.buffer[self.index]
  }

  pub fn frames(&self) -> &[f32; 600] {
    &self.buffer
  }
}

impl Default for FPSCounter {
  fn default() -> FPSCounter {
    FPSCounter::new()
  }
}


pub trait CanBeEmpty {
  /// Return the thing only if it is not empty.
  fn non_empty(&self) -> Option<&Self>;
}


impl CanBeEmpty for String {
  fn non_empty(&self) -> Option<&String> {
    if self.is_empty() { None } else { Some(self) }
  }
}

/// Clamp a number between two numbers
pub fn clamp<N: PartialOrd>(mn: N, n: N, mx: N) -> N {
  if n < mn {
    mn
  } else if n > mx {
    mx
  } else {
    n
  }
}
