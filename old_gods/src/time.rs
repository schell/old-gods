//! Because Instant::now() doesn't work on arch = wasm32.
#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;
pub use std::time::Duration;
#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[derive(Clone, Copy)]
pub struct Millis {
  #[cfg(target_arch = "wasm32")]
  millis: u32,

  #[cfg(not(target_arch = "wasm32"))]
  time: Instant
}


#[cfg(target_arch = "wasm32")]
impl Millis {
  pub fn now() -> Self {
    Millis {
      millis: window().unwrap().performance().unwrap().now() as u32
    }
  }

  pub fn millis_since(&self, then: Millis) -> u32 {
    self.millis - then.millis
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl Millis {
  pub fn now() -> Self {
    Millis {
      time: Instant::now()
    }
  }

  pub fn millis_since(&self, then: Millis) -> u32 {
    self.time.duration_since(then.time).as_millis() as u32
  }
}
