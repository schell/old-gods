//! Rendering data.
use specs::prelude::*;
use std::collections::{HashMap, HashSet};

use super::super::{
    prelude::{Color, FontDetails, V2},
    components::tiled::{Property, AABB as TiledAABB},
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


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// Various toggles to display or hide things during rendering.
/// Toggling the rendering of various debug infos can be done by adding a custom
/// property to your map file or individual objects.
pub enum RenderingToggles {
    /// Toggle marking actions.
    Actions,

    /// Toggle rendering positions.
    Positions,

    /// Toggle rendering barriers.
    Barriers,

    /// Toggle rendering the AABBTree.
    AABBTree,

    /// Toggle rendering velocities.
    Velocities,

    /// Toggle rendering zlevels.
    ZLevels,

    /// Toggle marking players.
    Players,

    /// Toggle marking the screen
    Screen,

    /// Toggle displaying the FPS.
    FPS,

    /// Render zones
    Zones,

    /// Fences
    Fences,

    /// Display the apparent entity count
    EntityCount,

    /// Display collision system information
    CollisionInfo,

    /// Display shapes
    Shapes,

    /// Display something else.
    /// Used for extension.
    Other(String),
}


/// Used for on-screen debugging of specific objects.
#[derive(Clone, Debug)]
pub struct ObjectRenderingToggles(pub HashSet<RenderingToggles>);


impl Component for ObjectRenderingToggles {
    type Storage = HashMapStorage<Self>;
}


impl RenderingToggles {
    pub fn property_map() -> HashMap<String, RenderingToggles> {
        use RenderingToggles::*;
        let props = vec![
            Actions,
            Positions,
            Barriers,
            AABBTree,
            Velocities,
            ZLevels,
            Players,
            Screen,
            FPS,
            Zones,
            Fences,
            EntityCount,
            CollisionInfo,
            Shapes,
        ];
        props
            .into_iter()
            .map(|t| (t.property_str().to_string(), t))
            .collect()
    }


    pub fn property_str(&self) -> &str {
        use RenderingToggles::*;
        match self {
            Actions => "toggle_rendering_actions",
            Positions => "toggle_rendering_positions",
            Barriers => "toggle_rendering_barriers",
            AABBTree => "toggle_rendering_aabb_tree",
            Velocities => "toggle_rendering_velocities",
            ZLevels => "toggle_rendering_z_levels",
            Players => "toggle_rendering_players",
            Screen => "toggle_rendering_screen",
            FPS => "toggle_rendering_fps",
            Zones => "toggle_rendering_zones",
            Fences => "toggle_rendering_fences",
            EntityCount => "toggle_rendering_entity_count",
            CollisionInfo => "toggle_rendering_collision_info",
            Shapes => "toggle_rendering_shapes",
            Other(s) => s,
        }
    }

    pub fn from_properties(props: &Vec<Property>) -> HashSet<RenderingToggles> {
        let toggles = Self::property_map();
        let mut set = HashSet::new();
        for prop in props {
            if !prop.name.starts_with("toggle_rendering_") {
                continue;
            }
            let toggle = toggles
                .get(&prop.name)
                .cloned()
                .unwrap_or(RenderingToggles::Other(prop.name.clone()));
            let should_set = prop.value.as_bool().unwrap_or(false);
            if should_set {
                set.insert(toggle.clone());
            }
        }
        set
    }

    pub fn remove_from_properties(
        props: &mut HashMap<String, Property>,
    ) -> Option<ObjectRenderingToggles> {
        let props_vec: Vec<Property> = props.iter().map(|(_, p)| p.clone()).collect();
        let toggles = Self::from_properties(&props_vec);
        if toggles.len() > 0 {
            for toggle in toggles.iter() {
                let _ = props.remove(toggle.property_str());
            }
            Some(ObjectRenderingToggles(toggles))
        } else {
            None
        }
    }
}
