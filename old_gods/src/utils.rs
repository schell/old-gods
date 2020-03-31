//! Utilities

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
