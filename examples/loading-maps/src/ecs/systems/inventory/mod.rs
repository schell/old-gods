/// Controls opening, closing and navigation of inventories.
use specs::prelude::*;
use std::collections::{HashMap, HashSet};

use old_gods::{
  prelude::{
    AABBTree, Action, Exile, FitnessStrategy, Lifespan, Name, Object,
    OriginOffset, Player, PlayerControllers, Position, Rendering, Shape,
    SuspendPlayer, AABB, JSON, V2,
  },
  utils::clamp,
};

mod inventory;
pub use inventory::*;


/// To facilitate "trade".
// TODO: Change Looting to Loot.
pub struct Looting {
  /// The inventory being looted.
  /// A value of 'None' means the looter is looting themselves.
  pub inventory: Option<Entity>,

  /// The entity looting the inventory
  pub looter: Entity,

  /// Is the looter looking in their own inventory, or someone else's?
  pub is_looking_in_own_inventory: bool,

  /// The index of the that the looter is currently looking at, if possible to
  /// determine (it's impossible if there are no items).
  pub index: Option<usize>,
}


impl Looting {
  pub fn clamp_index(&mut self, items_len: usize) {
    self.index = if items_len > 0 {
      self.index.map(|ndx| clamp(0, ndx, items_len - 1))
    } else {
      None
    };
  }

  pub fn pred_index(&mut self, items_len: usize) {
    self.index = if items_len > 0 {
      self.index.map(|ndx| {
        if ndx > 0 {
          clamp(0, ndx - 1, items_len - 1)
        } else {
          0
        }
      })
    } else {
      None
    }
  }

  pub fn succ_index(&mut self, items_len: usize) {
    self.index = if items_len > 0 {
      self.index.map(|ndx| clamp(0, ndx + 1, items_len - 1))
    } else {
      None
    };
  }
}


impl Component for Looting {
  type Storage = HashMapStorage<Looting>;
}


#[derive(Clone, PartialEq)]
pub enum InventoryAction {
  None,
  Use {
    inv: Entity,
    item_ndx: usize,
  },
  Drop {
    inv: Entity,
    item_ndx: usize,
  },
  Take {
    from: Entity,
    item_ndx: usize,
    to: Entity,
  },
}


#[derive(SystemData)]
pub struct LootingData<'a> {
  players: ReadStorage<'a, Player>,
  positions: ReadStorage<'a, Position>,
  entities: Entities<'a>,
  exiles: WriteStorage<'a, Exile>,
  items: WriteStorage<'a, Item>,
  lazy: Read<'a, LazyUpdate>,
  names: ReadStorage<'a, Name>,
  offsets: WriteStorage<'a, OriginOffset>,
  shapes: ReadStorage<'a, Shape>,
  suspensions: WriteStorage<'a, SuspendPlayer>,
  player_controllers: Read<'a, PlayerControllers>,
}


pub struct InventorySystem;


impl InventorySystem {
  /// Find actionless items on the map.
  pub fn find_actionless_map_items<'a>(
    entities: &Entities<'a>,
    items: &ReadStorage<'a, Item>,
    positions: &WriteStorage<'a, Position>,
    names: &ReadStorage<'a, Name>,
    exiles: &WriteStorage<'a, Exile>,
    actions: &WriteStorage<'a, Action>,
  ) -> Vec<(Entity, Name)> {
    // Items that have a position but no action need to have an action created
    // for them so they can be picked up.
    // Items that don't have a position are assumed to be sitting in an
    // inventory, and nothing has to be done.
    (entities, items, positions, names, !exiles, !actions)
      .join()
      .map(|(ent, _, _, name, _, _)| (ent, name.clone()))
      .collect()
  }


  /// Creates a new item pickup action
  pub fn new_pickup_action(
    &self,
    entities: &Entities,
    lazy: &LazyUpdate,
    name: String,
    p: V2,
    item_shape: Option<&Shape>,
  ) -> Entity {
    let a = Action {
      elligibles: vec![],
      taken_by: vec![],
      text: format!("Pick up {}", name),
      strategy: FitnessStrategy::HasInventory,
      lifespan: Lifespan::Many(1),
    };
    let s = item_shape
      .map(|s| {
        let aabb = s.aabb();
        let mut new_aabb = aabb.clone();
        new_aabb.extents += V2::new(4.0, 4.0);
        new_aabb.set_center(&aabb.center());
        new_aabb.to_shape()
      })
      .unwrap_or(Shape::Box {
        lower: V2::origin(),
        upper: V2::new(15.0, 15.0),
      });

    println!("Creating an action {:?}", a.text);

    lazy
      .create_entity(&entities)
      .with(a)
      .with(Position(p))
      .with(s)
      .with(Name("pickup item".to_string()))
      .build()
  }

  /// Manages a "trade" between two inventories (or one with itself), navigated
  /// by one Entity.
  /// Returns whether or not the "trade" is done, along with any inventory owning
  /// entities involved with a looting.
  pub fn run_looting(
    &self,
    looting: &mut Looting,
    inventories: &mut WriteStorage<Inventory>,
    data: &mut LootingData,
  ) -> (bool, Vec<Entity>) {
    let looter = looting.looter;
    let inventory = looting.inventory.unwrap_or(looter.clone());
    if looter == inventory {
      looting.is_looking_in_own_inventory = true;
      looting.inventory = None;
    }
    // Suspend character control for this entity, so that it can control
    // the looting process.
    let player = data
      .players
      .get(looter)
      .expect("TODO: Support looting for npcs.");
    let (navigated_ent, other_ent) = if looting.is_looking_in_own_inventory {
      (looter, inventory)
    } else {
      (inventory, looter)
    };
    let looting_ents = vec![navigated_ent.clone(), other_ent.clone()];
    // Determine if this is the first frame of the looting - if so it may not
    // have suspended controls and the buttons that are pressed this frame that
    // inserted the loot may still be down. We don't want to start the loot AND
    // start picking up items at the same time so we'll delay a frame.
    let is_control_suspended = data.suspensions.contains(looter);
    if !is_control_suspended {
      println!(
        "First frame of looting - suspending control for {:?}",
        data.names.get(looter)
      );
      data
        .suspensions
        .insert(looter, SuspendPlayer)
        .expect("Could not insert SuspendPlayer in looting system.");
      return (false, looting_ents);
    }

    let mut inv_action = InventoryAction::None;
    let result = {
      let navigated_inv = inventories
        .get(navigated_ent)
        .expect("Something is Trying to loot without an inventory");
      let other_inv = inventories
        .get(other_ent)
        .expect("Trying to loot an inventory that doesn't exist");
      let navigated_items_len = navigated_inv.items.len();
      let other_items_len = other_inv.items.len();
      // Make sure the index is up to date
      looting.clamp_index(navigated_items_len);
      // Set the index if we think this is the first time it has been opened.
      if looting.index.is_none() {
        // Maybe this is the first frame of the looting.
        if navigated_items_len > 0 {
          looting.index = Some(0);
        } else if other_items_len > 0 {
          // Toggle the navigation and try again
          println!(
            "Switching loot navigation to other inventory - the current one \
             is out of items."
          );
          looting.is_looking_in_own_inventory =
            !looting.is_looking_in_own_inventory;
          return self.run_looting(looting, inventories, data);
        } else {
          // There's absolutely nothing to do here, there are no items in either
          // inventory.
        }
      }

      // Navigate the looting
      let cloned_looting_ents = looting_ents.clone();
      let may_navigated_ent_name = data.names.get(navigated_ent);
      data
        .player_controllers
        .with_player_controller_at(player.0, |ctrl| {
          // If the user hits left or right, switch inventories if possible
          let can_look_in_own_inventory =
            !looting.is_looking_in_own_inventory && other_items_len > 0;
          let can_look_in_other_inventory =
            looting.is_looking_in_own_inventory && other_items_len > 0;
          if ctrl.left().is_on_this_frame() && can_look_in_own_inventory {
            looting.clamp_index(other_items_len);
            looting.is_looking_in_own_inventory = true;
          } else if ctrl.right().is_on_this_frame()
            && can_look_in_other_inventory
          {
            looting.clamp_index(other_items_len);
            looting.is_looking_in_own_inventory = false;
          }
          // Move the cursor up or down in the navigated inventory.
          let up = ctrl.up();
          let down = ctrl.down();
          if up.is_on_this_frame() || up.has_repeated_this_frame() {
            looting.pred_index(navigated_items_len);
          } else if down.is_on_this_frame() || down.has_repeated_this_frame() {
            looting.succ_index(navigated_items_len);
          }
          // Track whether the navigated inv even has an index. It won't if it has
          // no items
          let looting_has_index = looting.index.is_some();


          if looting_has_index {
            // Determine the item the looter is looking at
            let item_ndx = looting.index.unwrap() as usize;
            // Determine where the looter is going to put the item - if the looter
            // * is hitting A it means they want to trade the item
            // * is hitting B they want to drop the item onto the map
            // * is hitting X they want to use the item
            if ctrl.a().is_on_this_frame() {
              // Put this item in the other inventory.
              inv_action = InventoryAction::Take {
                from: navigated_ent,
                to: other_ent,
                item_ndx,
              }
            } else if ctrl.b().is_on_this_frame() {
              // Put this item on the map
              inv_action = InventoryAction::Drop {
                inv: navigated_ent,
                item_ndx,
              }
            } else if ctrl.x().is_on_this_frame() {
              // Use this item
              inv_action = InventoryAction::Use {
                inv: navigated_ent,
                item_ndx,
              }
            }
          };

          // If we haven't already returned, return whether or not the player is
          // hitting the inventory button (opens and closes the inventory)
          let done = ctrl.y().is_on_this_frame();
          if done {
            println!("Looting is done!");
          }
          (done, looting_ents)
        })
        .unwrap_or(
          // The controller must have been unplugged
          (true, cloned_looting_ents),
        )
    };

    match inv_action {
      InventoryAction::None => {}
      InventoryAction::Take { item_ndx, from, to } => {
        let item = inventories
          .get_mut(from)
          .map(|inv| inv.items.remove(item_ndx))
          .expect("could not get item from index");

        let into_inv = inventories
          .get_mut(to)
          .expect("could not get inventory to place item into");
        into_inv.add_item(item);
      }
      InventoryAction::Drop {
        item_ndx,
        inv: inv_ent,
      } => {
        let inv = inventories
          .get_mut(inv_ent)
          .expect("could not remove item from inventory");
        let loc = data
          .positions
          .get(inv_ent)
          .map(|p| p.0)
          .expect("tried to drop an item but the dropper has no position");
        let from_aabb = data
          .shapes
          .get(inv_ent)
          .map(|s| s.aabb())
          .unwrap_or(AABB::identity());
        inv.throw_item_with_index_onto_the_map(
          item_ndx,
          loc,
          from_aabb,
          &data.entities,
          &data.lazy,
        );
      }
      InventoryAction::Use { .. } => {
        panic!("TODO: use item");
      }
    }

    result
  }
}


/// In order for an item to be picked up it must have an associated Action.
/// The map loader creates this action automatically for all items.
/// When this action is taken, the item is placed into the taker's inventory.
/// Then the item is exiled.
impl<'a> System<'a> for InventorySystem {
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Write<'a, AABBTree>,
    WriteStorage<'a, Inventory>,
    WriteStorage<'a, JSON>,
    WriteStorage<'a, Looting>,
    WriteStorage<'a, Object>,
    ReadStorage<'a, Rendering>,
    LootingData<'a>,
  );

  fn run(
    &mut self,
    (
      entities,
      lazy,
      aabb_tree,
      mut inventories,
      mut jsons,
      mut lootings,
      mut objects,
      renderings,
      mut looting_data,
    ): Self::SystemData,
  ) {
    // Find any objects with inventory or item types so we can create the
    // items and Inventories.
    // Delete the object component afterward, if found.
    let mut invs = HashMap::new();
    let mut remove_objects = vec![];
    for (ent, obj) in (&entities, &objects).join() {
      match obj.type_is.as_str() {
        "inventory" => {
          remove_objects.push(ent);
          if obj.name.is_empty() {
            panic!("inventory must have a name");
          }
          // We have to have the items to put into the inv first, so we just store
          // this to process it later.
          invs.insert(obj.name.clone(), ent);
        }

        "item" => {
          remove_objects.push(ent);

          let properties = obj.json_properties();
          let rendering = renderings.get(ent).expect("item has no rendering");
          let shape = looting_data
            .shapes
            .get(ent)
            .cloned()
            .unwrap_or(Shape::box_with_size(0.0, 0.0));
          let offset = looting_data.offsets.get(ent).cloned();
          let item = Item {
            name: obj.name.clone(),
            usable: properties
              .get("usable")
              .map(|v| v.as_bool())
              .flatten()
              .unwrap_or(false),
            stack: properties
              .get("stack")
              .map(|v| v.as_u64().map(|u| u as usize))
              .flatten(),
            rendering: rendering.clone(),
            shape,
            offset
          };

          let _ = looting_data.items.insert(ent, item);
        }

        _ => {}
      }
    }
    // Remove the objects we created items and invs for.
    remove_objects.into_iter().for_each(|ent| {
      let _ = objects.remove(ent);
    });

    // Find the inventory holders (things that have an inventory), create the
    // inventories, add items to them, remove those items from the map, resolve
    // the inventory by name (or error) adding the inventory to the found
    // entity, and lastly delete the previous inventory entity.
    for (holder_ent, JSON(properties)) in (&entities, &mut jsons).join() {
      if let Some(name) = properties
        .remove("inventory_name")
        .map(|v| v.as_str().map(|s| s.to_string()))
        .flatten()
      {
        let inv_ent = invs.remove(&name).expect(&format!(
          "inventory_name must reference an existing inventory: cannot find \
           inventory named '{}'",
          name
        ));
        // The inventory should already have a shape from the TiledSystem,
        // so we can use it to query for any items that may be intersecting, and
        // then place those in the inventory.
        let items: Vec<Item> = aabb_tree
          .query_intersecting_shapes(
            &entities,
            &inv_ent,
            &looting_data.shapes,
            &looting_data.positions,
          )
          .into_iter()
          .filter_map(|(ent, _, _)| {
            if let Some(item) = looting_data.items.remove(ent) {
              entities
                .delete(ent)
                .expect("could not delete inventory item entity");
              Some(item)
            } else {
              None
            }
          })
          .collect();

        let inventory = Inventory::new(items);
        inventories
          .insert(holder_ent, inventory)
          .expect("could not create a new inventory");
        entities
          .delete(inv_ent)
          .expect("could not delete inventory object entity");
      }
    }

    if invs.len() > 0 {
      warn!("unclaimed inventories:\n'{:#?}'", invs.keys());
    }

    // run all looting
    let mut dead_loots = vec![];
    let mut looted_inventory_ents: HashSet<Entity> = HashSet::new();
    for (ent, mut looting) in (&entities, &mut lootings).join() {
      // run the looting
      let (looting_is_done, looting_ents) =
        self.run_looting(&mut looting, &mut inventories, &mut looting_data);
      // mark the looting as done if needed
      if looting_is_done {
        dead_loots.push(ent);

        // Remove suspend controls
        looting_ents.iter().for_each(|ent| {
          lazy.remove::<SuspendPlayer>(*ent);
        });
      }
      // add the involved looted inv entities to our set
      looted_inventory_ents.extend(looting_ents);
    }

    // Run through any toon inventories that are not in the set of ones already
    // involved in a looting
    for (ent, _inv, player, _) in (
      &entities,
      &mut inventories,
      &looting_data.players,
      !&looting_data.exiles,
    )
      .join()
    {
      let entity_already_looting_or_being_looted =
        looted_inventory_ents.contains(&ent);
      looting_data.player_controllers.with_player_controller_at(
        player.0,
        |ctrl| {
          // An entity can be looted without wanting to, so they need to be
          // able to shut that shit down!
          let inv_btn_on = ctrl.y().is_on_this_frame();
          let wants_to_open =
            inv_btn_on && !entity_already_looting_or_being_looted;
          let wants_to_close =
            inv_btn_on && entity_already_looting_or_being_looted;
          if wants_to_open {
            // Create a looting for it
            let _looting = lazy
              .create_entity(&entities)
              .with(Looting {
                inventory: Some(ent),
                looter: ent,
                // it's all their own inventory here!
                is_looking_in_own_inventory: true,
                index: None,
              })
              .build();
          } else if wants_to_close {
            // Search through all the lootings and find the one
            let loot_ent: Entity = *(&entities, &lootings)
              .join()
              .filter_map(|(loot_ent, loot)| {
                if loot.inventory == Some(ent) || loot.looter == ent {
                  Some(loot_ent)
                } else {
                  None
                }
              })
              .collect::<Vec<_>>()
              .first()
              .expect(&format!(
                "Player {:?} is trying to cancel a loot that doesn't exist",
                looting_data.names.get(ent)
              ));
            // Include it to be removed
            if !dead_loots.contains(&loot_ent) {
              dead_loots.push(loot_ent);
            }
          }
        },
      );
    }

    // destroy all the finished lootings
    dead_loots.into_iter().for_each(|ent| {
      entities.delete(ent).unwrap();
    });
  }
}
