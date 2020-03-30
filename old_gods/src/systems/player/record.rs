use ::specs::prelude::*;

use super::super::super::components::{Attributes, Name, ZLevel};
use super::super::super::geom::V2;
use super::super::super::tiled::json::{Object, Tiledmap};
use super::super::physics::Velocity;


/// A component for designating the maximum velocity of an entity.
#[derive(Clone, Debug)]
pub struct MaxSpeed(pub f32);


impl MaxSpeed {
  pub fn tiled_key() -> String {
    "max_speed".to_string()
  }
}


impl Component for MaxSpeed {
  type Storage = HashMapStorage<MaxSpeed>;
}


#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
/// All the AIs in our game.
pub enum AI {
  /// An AI that just walks left.
  WalksLeft,
}


#[derive(Debug, Clone, PartialEq, Hash, Eq)]
/// A player, controlled by an sdl controller.
pub struct Player(pub u32);


impl Player {
  pub fn tiled_key() -> String {
    "control".to_string()
  }

  pub fn get_entity<'a>(
    &self,
    entities: &Entities<'a>,
    players: &ReadStorage<'a, Player>,
  ) -> Option<Entity> {
    for (entity, player) in (entities, players).join() {
      if player == self {
        return Some(entity);
      }
    }
    None
  }
}


impl Component for Player {
  type Storage = HashMapStorage<Self>;
}


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
