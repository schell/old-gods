use specs::prelude::*;

use super::super::prelude::{
  Barrier, Easing, Exile, Inventory, Looting, Name, Position, Shape, Sprite,
  TweenParam, Velocity, AABB, V2,
};
use super::super::systems::tween;


/// Effects are used to take care of some of the more frequent and
/// nuanced interactions in the world, such as interacting with inventories
/// and changing sprite keyframes.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
  /// Change the keyframe of the given sprite.
  ChangeKeyframe {
    /// The sprite to change th keyframe of.
    sprite: Entity,

    /// The keyframe to change to.
    to: String,
  },

  /// Insert an item into an inventory
  InsertItem {
    item: Entity,
    into: Option<Entity>,
    from: Option<Entity>,
  },

  /// Reference an item and have some system turn this into an InsertItem effect
  /// at some later point.
  // TODO: Remove TakeItemLater.
  // Replace by having `into` be Option<Entity> and item be `Either<String, Entity>`.
  TakeItemLater { item_name: String },

  /// Use an item. Whatever that means. Generally this effect just sits and is.
  /// It should be handled by some system other than the EffectSystem.
  UseItem {
    /// The item being used
    item: Entity,

    /// The entity that invoked the item's use
    invoked_by: Entity,

    /// The inventory this item was used out of
    from: Entity,
  },

  /// Loot an inventory
  LootInventory {
    /// The inventory being looted
    inventory: Option<Entity>,

    /// The entity looting the inventory
    looter: Entity,
  },
}


impl Component for Effect {
  type Storage = HashMapStorage<Effect>;
}


/// Handles executing effects. Other systems inject effects and this system
/// processes and removes them.
pub struct EffectSystem;


impl<'a> System<'a> for EffectSystem {
  type SystemData = (
    ReadStorage<'a, Barrier>,
    ReadStorage<'a, Effect>,
    Entities<'a>,
    ReadStorage<'a, Exile>,
    WriteStorage<'a, Inventory>,
    Read<'a, LazyUpdate>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, Shape>,
    WriteStorage<'a, Sprite>,
    WriteStorage<'a, Velocity>,
  );

  fn run(
    &mut self,
    (
      _barriers,
      effects,
      entities,
      exiles,
      mut inventories,
      lazy,
      names,
      mut positions,
      shapes,
      mut sprites,
      mut velocities,
    ): Self::SystemData,
  ) {
    let remove_item_from = |
      inventories: &mut WriteStorage<Inventory>,
      item: &Entity,
      from: &Entity,
    | {
      // Remove the item from the previous inventory, if possible
      let from_inventory =
        inventories
        .get_mut(*from)
        .expect("Found a remove item effect referencing something that doesn't have an inventory!");
      let ndx =
        from_inventory
          .items
          .iter()
          .position(|i| i == item)
          .expect(&format!(
            "Inventory {:?} does not contain {:?}!\nContains: {:?}",
            names.get(*from),
            (item, names.get(*item)),
            from_inventory
              .items
              .iter()
              .map(|item| {
                names
                  .get(*item)
                  .map(|n| (item, n.0.clone()))
                  .unwrap_or((item, "unnamed".to_string()))
              })
              .collect::<Vec<_>>()
          ));
      from_inventory.items.remove(ndx);
      println!(
        "{:?} no longer holds {:?}",
        names.get(*from),
        names.get(*item)
      );
    };

    let insert_item_into = |
      inventories: &mut WriteStorage<Inventory>,
      positions: &mut WriteStorage<Position>,
      item: &Entity,
      into: &Entity,
    | {
      // Add the item to the target inventory
      let into_inventory =
          inventories
          .get_mut(*into)
          .expect("Found an insert item effect referencing something that doesn't have an inventory!");
      into_inventory.items.push(*item);
      // Remove the item from the map if possible
      positions.remove(*item);
      println!("{:?} now holds {:?}", names.get(*into), names.get(*item));
    };

    for (ent, effect, ()) in (&entities, &effects, !&exiles).join() {
      match effect {
        Effect::ChangeKeyframe {
          sprite: sprite_ent,
          to: keyframe,
        } => {
          let sprite = sprites
            .get_mut(*sprite_ent)
            .expect("Could not find a sprite!");
          println!(
            "Changing keyframe of {:?} from {:?} to {:?}",
            names.get(*sprite_ent),
            sprite.keyframe,
            keyframe
          );
          sprite.keyframe = Some(keyframe.clone());
          entities.delete(ent).expect("Could not delete an effect");
        }

        // Inserting from the map into an inventory
        Effect::InsertItem {
          item,
          into: Some(into),
          from: None,
          ..
        } => {
          insert_item_into(&mut inventories, &mut positions, item, into);
          entities.delete(ent).expect("Could not delete an effect");
        }

        // Inserting from one inventory into another
        Effect::InsertItem {
          item,
          into: Some(into),
          from: Some(from),
          ..
        } => {
          insert_item_into(&mut inventories, &mut positions, item, into);
          remove_item_from(&mut inventories, item, from);
          entities.delete(ent).expect("Could not delete an effect");
        }

        // Dropping an item from an inventory onto the map
        Effect::InsertItem {
          item,
          into: None,
          from: Some(from),
        } => {
          remove_item_from(&mut inventories, item, from);
          // Give the item a position on the map
          let mut loc: V2 = positions
            .get(*from)
            .map(|p| p.0)
            .expect("Tried to drop an item but the dropper has no position!");
          // Find a position around the inventory that's out of the way
          let from_aabb = shapes
            .get(*from)
            .map(|s| s.aabb())
            .unwrap_or(AABB::identity());
          let item_aabb = shapes
            .get(*item)
            .map(|s| s.aabb())
            .unwrap_or(AABB::identity());
          // From there we must offset it some amount to account for
          // the barriers of each
          let radius = {
            let f = from_aabb.greater_extent();
            let i = item_aabb.greater_extent();
            f32::max(f, i)
          };

          // Place the item
          let inventory = inventories.get_mut(*from).expect(
            "Attempting to remove an item from an inventory that doesn't exist",
          );
          let radians = inventory.dequeue_ejection_in_radians();
          let dv = V2::new(f32::cos(radians), f32::sin(radians));
          loc = loc + (dv.scalar_mul(radius));
          positions
            .insert(*item, Position(loc))
            .expect("Could not insert a Position");

          // Fuckit! Throw the item!
          let speed = 100.0;
          let starting_v = dv.scalar_mul(speed);
          velocities
            .insert(*item, Velocity(starting_v))
            .expect("Could not insert a Velocity");
          println!("dv:{:?} radius:{:?} vel:{:?}", dv, radius, starting_v);
          // Tween the item flying out of the inventory, eventually stopping.
          tween::tween(
            &entities,
            *item,
            &lazy,
            TweenParam::Velocity(starting_v, V2::origin()),
            Easing::Linear,
            0.5,
          );

          // This effect is now dead
          entities.delete(ent).expect("Could not delete an effect");
        }

        // This effect is waiting for some more data...
        Effect::InsertItem {
          into: None,
          from: None,
          ..
        } => {}

        Effect::LootInventory { inventory, looter } => {
          let inventory: Option<Entity> = if let Some(inventory) = inventory {
            if inventory == looter {
              None
            } else {
              Some(*inventory)
            }
          } else {
            *inventory
          };
          lazy
            .create_entity(&entities)
            .with(Looting {
              inventory: inventory,
              looter: *looter,
              is_looking_in_own_inventory: inventory.is_none(),
              index: None,
            })
            .build();
          entities.delete(ent).expect("Could not delete an effect");
        }

        Effect::TakeItemLater { .. } => {}

        // This should be handled by some other system...
        Effect::UseItem { .. } => {}
      }
    }
  }
}
