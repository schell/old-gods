//! Rendering data.
use specs::prelude::*;

use super::super::{
    color::Color, components::FontDetails, geom::*, tiled::json::AABB as TiledAABB,
};

/// ## ZLevel
/// Determines rendering order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZLevel(pub f32);


impl Component for ZLevel {
    type Storage = VecStorage<ZLevel>;
}


impl ZLevel {
    /// Adds an amount and returns a new zlevel.
    pub fn add(&self, n: f32) -> ZLevel {
        ZLevel(self.0 + n)
    }
}


/// Helps render tiles by allowing an origin offset during z-sorting and
/// rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginOffset(pub V2);


impl OriginOffset {
    pub fn tiled_type() -> String {
        "origin_offset".to_string()
    }
}


impl Component for OriginOffset {
    type Storage = DenseVecStorage<OriginOffset>;
}


/// ## ActionButton
/// These are buttons on the user's controller that show them they can take an
/// action on an object.
#[derive(Debug, Clone)]
pub enum ActionButton {
    A,
    B,
    X,
    Y,
}


#[derive(Debug, Clone, PartialEq, Hash)]
/// A frame within a texture.
pub struct TextureFrame {
    /// The name of the sprite sheet.
    pub sprite_sheet: String,

    /// The source rectangle within the spritesheet.
    pub source_aabb: TiledAABB<u32>,

    /// The destination size
    pub size: (u32, u32),

    pub is_flipped_horizontally: bool,

    pub is_flipped_vertically: bool,

    pub is_flipped_diagonally: bool,
}


impl TextureFrame {
    pub fn scale(&self) -> V2 {
        let sx = self.size.0 as f32 / self.source_aabb.w as f32;
        let sy = self.size.1 as f32 / self.source_aabb.h as f32;
        V2::new(sx, sy)
    }
}


#[derive(Debug, Clone, PartialEq, Hash)]
/// Drawn text.
pub struct Text {
    pub text: String,
    pub font: FontDetails,
    pub color: Color,
    pub size: (u32, u32),
}


impl Text {
    pub fn as_key(&self) -> String {
        format!("{:?}", self)
    }
}


/// The base types that can be rendered
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum RenderingPrimitive {
    TextureFrame(TextureFrame),
    Text(Text),
}


/// A composite rendering type representing a display list
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Rendering {
    /// The alpha mod of this rendering
    pub alpha: u8,

    /// The primitive of this rendering
    pub primitive: RenderingPrimitive,
}


impl Rendering {
    pub fn from_frame(frame: TextureFrame) -> Rendering {
        Rendering {
            alpha: 255,
            primitive: RenderingPrimitive::TextureFrame(frame),
        }
    }

    pub fn from_text(text: Text) -> Rendering {
        Rendering {
            alpha: 255,
            primitive: RenderingPrimitive::Text(text),
        }
    }

    pub fn as_frame(&self) -> Option<&TextureFrame> {
        match &self.primitive {
            RenderingPrimitive::TextureFrame(t) => Some(t),
            _ => None,
        }
    }


    pub fn size(&self) -> (u32, u32) {
        match &self.primitive {
            RenderingPrimitive::TextureFrame(t) => t.size,
            RenderingPrimitive::Text(t) => t.size,
        }
    }
}


impl Component for Rendering {
    type Storage = DenseVecStorage<Self>;
}
