/// # Inventory
/// An inventory is a container of items.
/// This module provides the components and records needed to parse inventories
/// from a Tiled map and insert them into the ECS.
use specs::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;

use super::super::super::components::{Attribute, Item, Name, ZLevel};
use super::super::super::geom::{AABB, V2};
use super::super::super::tiled::json::{Object, Tiledmap};
use super::super::super::utils::CanBeEmpty;
use super::super::item::ItemRecord;


const ITEM_PLACEMENTS: [f32; 16] = [
  0.0,
  PI / 2.0,
  PI,
  3.0 * PI / 2.0,
  PI / 4.0,
  3.0 * PI / 4.0,
  5.0 * PI / 4.0,
  7.0 * PI / 4.0,
  PI / 6.0,
  PI / 3.0,
  2.0 * PI / 3.0,
  5.0 * PI / 6.0,
  7.0 * PI / 6.0,
  4.0 * PI / 3.0,
  5.0 * PI / 3.0,
  11.0 * PI / 6.0,
];

/// An entity with an inventory can store items.
#[derive(Debug, Clone)]
pub struct Inventory {
  /// The items that are inside this inventory.
  pub items: Vec<Entity>,

  /// A place to store the next angle to use for throwing an item out
  /// of the inventory.
  pub next_ejection_angle: u32,
}


impl Inventory {
  pub fn new(items: Vec<Entity>) -> Inventory {
    Inventory {
      items,
      next_ejection_angle: 0,
    }
  }

  pub fn tiled_key_name() -> String {
    "inventory_name".to_string()
  }

  pub fn remove_item(&mut self, item: &Entity) -> Result<(), String> {
    let mut may_ndx = None;
    for (item_here, ndx) in self.items.iter().zip(0..) {
      if *item_here == *item {
        may_ndx = Some(ndx);
        break;
      }
    }
    let ndx = may_ndx.ok_or("Could not find item")?;
    self.items.remove(ndx);
    Ok(())
  }

  /// Dequeue the next item ejection angle. This is nice for
  /// a good item dropping effect.
  pub fn dequeue_ejection_in_radians(&mut self) -> f32 {
    let n = self.next_ejection_angle as usize;
    self.next_ejection_angle += 1;

    ITEM_PLACEMENTS[n % ITEM_PLACEMENTS.len()]
  }
}


impl Inventory {
  /// Run the frame by frame upkeep on an inventory.
  /// This stacks items.
  pub fn upkeep(
    &mut self,
    entities: &Entities,
    items: &mut WriteStorage<Item>,
    names: &ReadStorage<Name>,
  ) {
    // A hashmap that holds the entity and stack count
    let mut m: HashMap<String, (Entity, usize)> = HashMap::new();

    // The new vec of items
    let mut new_items = vec![];

    for ent in &self.items {
      let name = names.get(*ent).expect("An item doesn't have a name");
      let item = items
        .get(*ent)
        .expect(&format!("An item named {:?} is not an Item", name));

      if item.stack.is_some() {
        if item.stack.unwrap() == 0 {
          // This item doesn't exist, it's a stack of zero
          entities.delete(*ent).unwrap();
        } else {
          // TODO: Inspect what happens when two different items with the same name come
          // in from a Tiled map (looks like they're stacking without stack_count defined)
          if let Some(mut entry) = m.get_mut(&name.0) {
            // We've already seen this item and it's stackable, so stack them!
            entry.1 = item.stack.unwrap() + entry.1;
            // remove the old one that we've just added to the stack.
            entities.delete(*ent).unwrap();
            println!("Stacking {:?}", name.0);
          } else {
            m.insert(name.0.clone(), (*ent, item.stack.unwrap()));
            new_items.push(*ent);
          }
        }
      } else {
        // This item is not stackable
        m.insert(name.0.clone(), (*ent, 1));
        new_items.push(*ent);
      }
    }

    // update the inventory to our new items
    self.items = new_items;

    // go through our hash map and update the item counts
    m.drain().for_each(|(_, (ent, count))| {
      let item = items.get_mut(ent).unwrap();
      item.stack = Some(count);
    });
  }
}


impl Component for Inventory {
  type Storage = HashMapStorage<Inventory>;
}


/// All the bits needed to create a new inventory in the ECS.
/// This structure is parsed by the map loader. The map loader
/// collects this along with all the other record types and then
/// decomposes them into components and inserts them into the ECS.
#[derive(Clone)]
pub struct InventoryRecord {
  name: String,
  items: Vec<ItemRecord>,
  aabb: AABB,
}


impl<'a> InventoryRecord {
  /// Read an InventoryRecord from a TiledMap.
  /// This will not populate its items, see InventoryLayer.
  pub fn read(object: &Object) -> Result<InventoryRecord, String> {
    let name = object
      .name
      .non_empty()
      .ok_or("Inventory must have a name")?
      .clone();
    let aabb = AABB {
      top_left: V2::new(object.x as f32, object.y as f32),
      extents: V2::new(object.width as f32, object.height as f32),
    };
    Ok(InventoryRecord {
      name,
      aabb,
      items: vec![],
    })
  }

  /// Decompose into components and enter them into the ECS.
  pub fn into_ecs(
    self,
    world: &mut World,
    z: ZLevel,
  ) -> Result<Entity, String> {
    let items: Vec<Entity> = self
      .items
      .into_iter()
      .map(|i| i.into_ecs(world, z))
      .collect();

    let inventory = world.create_entity().with(Name(self.name.clone())).build();

    world
      .write_storage::<Inventory>()
      .insert(inventory, Inventory::new(items))
      .expect("Could not create an Inventory");

    Ok(inventory)
  }
}

/// All the bits needed to enter a whole layer of inventories into the ECS.
pub struct InventoryLayer {
  inventories: Vec<InventoryRecord>,
}


impl<'a> InventoryLayer {
  pub fn read(
    map: &Tiledmap,
    objects: &Vec<Object>,
  ) -> Result<InventoryLayer, String> {
    // Hold vecs for our records.
    let mut items: Vec<ItemRecord> = vec![];
    let mut invs: Vec<InventoryRecord> = vec![];

    // Read our records, then associate them later.
    for object in objects {
      match object.type_is.as_str() {
        "item" => {
          let rec = ItemRecord::read(map, object)?;
          items.push(rec);
        }
        "inventory" => {
          let rec = InventoryRecord::read(object)?;
          invs.push(rec);
        }
        ty => {
          return Err(format!(
            "Could not load unknown inventory layer object type {:?}",
            ty
          ));
        }
      }
    }

    // Associate the items with their inventories.
    for mut item in items {
      'item_inventories: for inv in &mut invs {
        if inv.aabb.contains_point(&item.layer_placement) {
          item.attributes.attribs = item
            .attributes
            .attribs
            .into_iter()
            .filter_map(|a| match a {
              Attribute::Position(_) => None,
              a => Some(a),
            })
            .collect();
          inv.items.push(item.clone());
          break 'item_inventories;
        }
      }
    }

    Ok(InventoryLayer { inventories: invs })
  }

  /// Decompose into comps and enter them into the ECS.
  pub fn into_ecs(
    self,
    world: &mut World,
    z: ZLevel,
  ) -> Result<Vec<Entity>, String> {
    self.inventories.into_iter().fold(Ok(vec![]), |res, i| {
      let mut res = res?;
      let ent = i.into_ecs(world, z)?;
      res.push(ent);
      Ok(res)
    })
  }
}
