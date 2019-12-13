//! Definitions of Color

/// A color.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8,
  a: u8
}


impl Color {
  pub fn RGB(r:u8, g:u8, b:u8) -> Color {
    Color {
      r, g, b, a: 255
    }
  }

  pub fn RGBA(r:u8, g:u8, b:u8, a:u8) -> Color {
    Color {
      r, g, b, a
    }
  }
}


/// A color used for the background
pub struct BackgroundColor(pub Color);


impl Default for BackgroundColor {
  fn default() -> Self {
    BackgroundColor(Color::RGB(0, 0, 0))
  }
}
