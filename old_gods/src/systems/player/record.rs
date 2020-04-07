use ::specs::prelude::*;

use super::super::super::components::{Attributes, Name, ZLevel};
use super::super::super::geom::V2;
use super::super::super::tiled::json::{Object, Tiledmap};
use super::super::physics::Velocity;


/// All the data needed from a Tiled map in order to create
/// a player.
pub struct ToonRecord {
  pub attributes: Attributes,
}


impl<'a> ToonRecord {
  /// Read a ToonRecord from a tiled Object.
  pub fn read(map: &Tiledmap, object: &Object) -> Result<ToonRecord, String> {
    let attributes = Attributes::read(map, object)?;
    let name: Name = attributes.name().ok_or("A player must have a name.")?;
    let _position = attributes
      .position()
      .ok_or("A player must have a position.")?;
    let _rendering_or_anime = attributes
      .rendering_or_anime()
      .ok_or("A player must have a rendering or animation.")?;
    let _control = attributes.control().ok_or(format!(
      "Player {} must have a 'control' custom property.",
      name.0
    ))?;
    Ok(ToonRecord { attributes })
  }

  /// Decompose an ToonRecord into components and enter them into
  /// the ECS.
  pub fn into_ecs(self, world: &mut World, z: ZLevel) -> Entity {
    let ent = self.attributes.into_ecs(world, z);
    world
      .write_storage()
      .insert(ent, Velocity(V2::new(0.0, 0.0)))
      .expect("Could not insert velocity.");
    ent
  }
}
