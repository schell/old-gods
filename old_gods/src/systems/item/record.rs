use ::specs::prelude::*;

use super::super::super::components::{Attributes, ZLevel};
use super::super::super::geom::V2;
use super::super::super::tiled::json::{Object, Tiledmap};


/// An entity with an item component can be kept in an inventory.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
  /// Whether or not this item is usable by itself.
  pub usable: bool,

  /// If this item can be stacked the Option type holds
  /// the count of the stack.
  pub stack: Option<usize>,
}


impl Component for Item {
  type Storage = HashMapStorage<Item>;
}


/// All the bits needed to create a new item in the ECS.
/// This structure is parsed by the map loader. The map loader
/// collects this along with all the other record types and then
/// decomposes them into components and inserts them into the ECS.
#[derive(Debug, Clone)]
pub struct ItemRecord {
  pub layer_placement: V2,
  pub attributes: Attributes,
}


impl<'a> ItemRecord {
  /// Read an ItemRecord out from a tiled Object.
  pub fn read(map: &Tiledmap, object: &Object) -> Result<ItemRecord, String> {
    let attributes = Attributes::read(map, object)?;
    let _name = attributes.name().ok_or("An item must have a name.")?;
    // Tiled tiles' origin is at the bottom of the tile, not the top
    let position = attributes.position().ok_or("This will never happen.")?;
    let layer_placement = position.0;
    let _rendering_or_anime = attributes
      .rendering_or_anime()
      .ok_or("An item must have a rendering or animation.")?;
    let _item = attributes
      .item()
      .ok_or("An Tiled object item must be an Old Gods Engine item")?;

    Ok(ItemRecord {
      attributes,
      layer_placement,
    })
  }

  /// Decompose an ItemRecord into components and enter them into
  /// the ECS.
  pub fn into_ecs(self, world: &mut World, z: ZLevel) -> Entity {
    self.attributes.into_ecs(world, z)
  }
}
