//! Definitions of Color

/// A color.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
}


impl Color {
  pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
  }

  pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
  }
}


/// A color used for the background
pub struct BackgroundColor(pub Color);


impl Default for BackgroundColor {
  fn default() -> Self {
    BackgroundColor(Color::rgb(0, 0, 0))
  }
}
