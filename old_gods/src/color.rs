//! Definitions of Color
use wasm_bindgen::JsValue;

pub mod css;

/// A color.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
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

    pub fn into_rgb(self) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.g,
            a: 255
        }
    }
}


impl From<&Color> for JsValue {
    fn from(color: &Color) -> JsValue {
        let s = format!(
            "rgba({}, {}, {}, {:.3})",
            color.r, color.g, color.b, (color.a as f32/ 255.0)
        );
        JsValue::from_str(&s)
    }
}


impl From<u32> for Color {
    fn from(n: u32) -> Color {
        Color::rgba(
            (n >> 24 & 0xff) as u8,
            (n >> 16 & 0xff) as u8,
            (n >> 8 & 0xff) as u8,
            (n & 0xff) as u8
        )
    }
}


/// A color used for the background
pub struct BackgroundColor(pub Color);


impl Default for BackgroundColor {
    fn default() -> Self {
        BackgroundColor(Color::rgb(0, 0, 0))
    }
}


#[cfg(test)]
mod color_tests {
    use super::*;
    use super::css::red;

    #[test]
    fn hex() {
        let css_red = red();
        let hex_red = Color::from(0xff0000ff);
        assert_eq!(css_red, hex_red);
    }
}
