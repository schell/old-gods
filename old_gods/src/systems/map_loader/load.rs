use std::iter::FromIterator;
use std::iter::Iterator;

use super::super::{
  super::tiled::json::*,
  super::tiled::json::AABB as TiledAABB,
  physics::*,
  animation::*,
  animation::Frame as AnimeFrame,
  super::components::{Rendering, TextureFrame},
  super::geom::{
    Shape,
    V2
  }
};


pub fn get_tile_aabb(tm: &Tiledmap, tile_gid: &GlobalId) -> Option<TiledAABB<u32>> {
  let (firstgid, tileset) = tm.get_tileset_by_gid(tile_gid)?;
  tileset.aabb(firstgid, tile_gid)
}


pub fn get_tile_position(
  tm: &Tiledmap,
  tile_gid: &GlobalId,
  ndx: usize
) -> Option<Position> {
  let (width, height) = (tm.width as u32, tm.height as u32);
  let yndx = ndx as u32 / width;
  let xndx = ndx as u32 % height;
  let aabb = get_tile_aabb(tm, tile_gid)?;
  Some(Position(V2::new(
    (xndx * aabb.w) as f32,
    (yndx * aabb.h) as f32,
  )))
}


pub fn get_tile_rendering_offset(t: &Tile) -> Option<V2> {
  t
    .object_with_type(&"rendering_origin_offset".to_string())
    .and_then(|o| Some(V2::new(o.x, o.y)))
}


/// Return a rendering for the tile with the given GlobalId.
pub fn get_tile_rendering(
  tm: &Tiledmap,
  gid: &GlobalTileIndex,
  size:Option<(u32, u32)>
) -> Option<Rendering> {
  let (firstgid, tileset) =
    tm
    .get_tileset_by_gid(&gid.id)?;
  let aabb =
    tileset
    .aabb(firstgid, &gid.id)?;
  Some(
    Rendering::from_frame(
      TextureFrame {
        sprite_sheet: tileset.image.clone(),
        source_aabb: aabb.clone(),
        size:
        size
          .unwrap_or((aabb.w, aabb.h)),
        is_flipped_horizontally: gid.is_flipped_horizontally,
        is_flipped_vertically: gid.is_flipped_vertically,
        is_flipped_diagonally: gid.is_flipped_diagonally,
      }
    )
  )
}


pub fn get_tile_animation(
  tm: &Tiledmap,
  gid: &GlobalTileIndex,
  size: Option<(u32, u32)>
) -> Option<Animation> {
  let (firstgid, tileset) =
    tm
    .get_tileset_by_gid(&gid.id)?;
  let tile =
    tileset
    .tile(firstgid, &gid.id)?;
  // Get out the animation frames
  let frames =
    tile
    .clone()
    .animation?;
  Some(
    Animation {
      is_playing: true,
      frames:
      Vec::from_iter(
        frames
          .iter()
          .filter_map(|frame| {
            tileset
              .aabb_local(&frame.tileid)
              .map(|frame_aabb| {
                let size =
                  size
                  .unwrap_or((frame_aabb.w, frame_aabb.h));
                AnimeFrame {
                  rendering:
                  Rendering::from_frame(
                    TextureFrame {
                      sprite_sheet: tileset.image.clone(),
                      source_aabb: frame_aabb.clone(),
                      size,
                      is_flipped_horizontally: gid.is_flipped_horizontally,
                      is_flipped_vertically: gid.is_flipped_vertically,
                      is_flipped_diagonally: gid.is_flipped_diagonally,
                    }
                  ),
                  duration: frame.duration as f32 / 1000.0
                }
              })
          })
      ),
      current_frame_index: 0,
      current_frame_progress: 0.0,
      should_repeat: true
    }
  )
}


/// Returns the first barrier aabb on the object.
pub fn get_tile_barriers(
  tm: &Tiledmap,
  tile_gid: &GlobalId
) -> Option<Shape> {
  if let Some(group) = tm.get_tile_object_group(&tile_gid) {
    for tile_object in group.objects {
      let may_bar = object_barrier(&tile_object);
      if may_bar.is_some() {
        return may_bar;
      }
    }
  }
  None
}


pub fn get_z_inc(object: &Object) -> Option<i32> {
  get_z_inc_props(&object.properties)
}

pub fn get_z_inc_props(properties: &Vec<Property>) -> Option<i32> {
  for prop in properties {
    if prop.name == "z" {
      let zinc =
        prop
        .value
        .as_i64()
        .expect("Could not deserialize z incement.") as i32;
      return Some(zinc);
    }
  }
  None
}


pub fn object_shape(object: &Object) -> Option<Shape> {
  if let Some(_polyline) = &object.polyline {
    // A shape cannot be a polyline
    None
  } else if let Some(polygon) = &object.polygon {
    let vertices:Vec<V2> =
      polygon
      .clone()
      .into_iter()
      .map(|p| V2::new(p.x + object.x, p.y + object.y))
      .collect();
    // TODO: Check polygon for concavity at construction
    // ```rust
    // pub fn polygon_from_vertices() -> Option<Shape>
    // ```
    // because not all polygons are convex

    // TODO: Support a shape made of many shapes.
    // This way we can decompose concave polygons into a number of convex ones.
    Some(
      Shape::Polygon {
        vertices
      }
    )
  } else {
    // It's a rectangle!
    let lower =
      V2::new(object.x, object.y);
    let upper =
      V2::new(object.x + object.width, object.y + object.height);
    Some(
      Shape::Box {
        lower,
        upper
      }
    )
  }
}


pub fn object_barrier(object: &Object) -> Option<Shape> {
  if object.type_is == "barrier" {
    object_shape(object)
  } else {
    None
  }
}
