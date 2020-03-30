/// Controls opening, closing and navigation of inventories.
use specs::prelude::*;
use std::collections::HashSet;

//use super::ui::{UI};
use super::super::components::{
  Action, Effect, Exile, Item, Name, Player, Position, SuspendPlayer,
};
use super::super::utils::clamp;

mod record;
pub use record::*;


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
  //Close,
  Use,
  Drop,
  Trade,
}


pub struct InventorySystem;


impl InventorySystem {
  /// Manages a "trade" between two inventories (or one with itself), navigated
  /// by one Entity.
  /// Returns whether or not the "trade" is done, along with any inventory owning
  /// entities involved with a looting.
  pub fn run_looting(
    &self,
    looting: &mut Looting,
    inventories: &mut WriteStorage<Inventory>,
    entities: &Entities,
    lazy: &LazyUpdate,
    names: &ReadStorage<Name>,
    positions: &ReadStorage<Position>,
    players: &ReadStorage<Player>,
    suspend_controls: &mut WriteStorage<SuspendPlayer>,
    //ui: &UI
  ) -> (bool, Vec<Entity>) {
    let looter = looting.looter;
    let inventory = looting.inventory.unwrap_or(looter.clone());
    if looter == inventory {
      looting.is_looking_in_own_inventory = true;
      looting.inventory = None;
    }
    // Suspend character control for this entity, so that it can control
    // the looting process.
    let _player = players
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
    let is_control_suspended = suspend_controls.contains(looter);
    if !is_control_suspended {
      println!(
        "First frame of looting - suspending control for {:?}",
        names.get(looter)
      );
      suspend_controls
        .insert(looter, SuspendPlayer)
        .expect("Could not insert SuspendPlayer in looting system.");
      return (false, looting_ents);
    }

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
          "Switching loot navigation to other inventory - the current one is out of items."
        );
        looting.is_looking_in_own_inventory =
          !looting.is_looking_in_own_inventory;
        return self.run_looting(
          looting,
          inventories,
          entities,
          lazy,
          names,
          positions,
          players,
          suspend_controls,
          //ui
        );
      } else {
        // There's absolutely nothing to do here, there are no items in either
        // inventory.
      }
    }

    // Navigate the looting
    //if let Some(ctrl) = ui.get_player_controller(player.0) {
    //  // If the user hits left or right, switch inventories if possible
    //  let can_look_in_own_inventory =
    //    !looting.is_looking_in_own_inventory
    //    && other_items_len > 0;
    //  let can_look_in_other_inventory =
    //    looting.is_looking_in_own_inventory
    //    && other_items_len > 0;
    //  if ctrl.left().is_on_this_frame() && can_look_in_own_inventory {
    //    looting.clamp_index(other_items_len);
    //    looting.is_looking_in_own_inventory = true;
    //  } else if ctrl.right().is_on_this_frame() && can_look_in_other_inventory {
    //    looting.clamp_index(other_items_len);
    //    looting.is_looking_in_own_inventory = false;
    //  }
    //  // Move the cursor up or down in the navigated inventory.
    //  let up = ctrl.up();
    //  let down = ctrl.down();
    //  if up.is_on_this_frame() || up.has_repeated_this_frame() {
    //    looting.pred_index(navigated_items_len);
    //  } else if down.is_on_this_frame() || down.has_repeated_this_frame() {
    //    looting.succ_index(navigated_items_len);
    //  }
    //  // Track whether the navigated inv even has an index. It won't if it has
    //  // no items
    //  let looting_has_index =
    //    looting
    //    .index
    //    .is_some();
    //  if looting_has_index {
    //    // Determine the item the looter is looking at
    //    let item_ndx =
    //      looting
    //      .index
    //      .unwrap() as usize;
    //    let item =
    //      navigated_inv
    //      .items
    //      .get(item_ndx)
    //      .expect(&format!(
    //        "No item in inventory {:?} at index {:?}\ninventory:\n{:?}",
    //        names.get(navigated_ent),
    //        item_ndx,
    //        navigated_inv
    //      ));
    //    // Looting will happen from the navigated inventory.
    //    let from =
    //      Some(navigated_ent);
    //    // Determine where the looter is going to put the item - if the looter
    //    // * is hitting A it means they want to trade the item
    //    // * is hitting B they want to drop the item onto the map
    //    // * is hitting X they want to use the item
    //    let (inv_action, into):(InventoryAction, Option<_>) =
    //      if ctrl.a().is_on_this_frame() {
    //        (InventoryAction::Trade, Some(other_ent))
    //      } else if ctrl.b().is_on_this_frame() {
    //        (InventoryAction::Drop, None)
    //      } else if ctrl.x().is_on_this_frame() {
    //        (InventoryAction::Use, None)
    //      } else {
    //        (InventoryAction::None, None)
    //      };
    //    // Create an effect to move the item from the navigated inventory into
    //    // the other inventory.
    //    let is_trading_or_dropping =
    //      inv_action == InventoryAction::Trade
    //        || inv_action == InventoryAction::Drop;
    //    if is_trading_or_dropping {
    //      let effect =
    //        Effect::InsertItem {
    //          item: *item,
    //          into,
    //          from
    //        };
    //      let _effect_ent =
    //        lazy
    //        .create_entity(entities)
    //        .with(effect)
    //        .build();
    //      // Adjust the looting index (the item list just went down by one)
    //      looting.clamp_index(navigated_items_len - 1);
    //      return (false, looting_ents);
    //    } else if inv_action == InventoryAction::Use {
    //      let invoked_by =
    //        looting.looter;
    //      let effect =
    //        Effect::UseItem {
    //          invoked_by,
    //          item: *item,
    //          from: from.unwrap()
    //        };
    //      let _effect_ent =
    //        lazy
    //        .create_entity(entities)
    //        .with(effect)
    //        .build();
    //      return (true, looting_ents);
    //    }
    //  }

    //  // If we haven't already returned, return whether or not the player is
    //  // hitting the inventory button (opens and closes the inventory)
    //  let done =
    //    ctrl.y().is_on_this_frame();
    //  if done {
    //    println!("Looting is done!");
    //  }
    //  return (
    //    done,
    //    looting_ents
    //  );
    //} else {
    //  // The controller must have been unplugged
    (true, looting_ents)
    //}
  }
}


/// In order for an item to be picked up it must have an associated Action.
/// The map loader creates this action automatically for all items.
/// When this action is taken, the item is placed into the taker's inventory.
/// Then the item is exiled.
impl<'a> System<'a> for InventorySystem {
  type SystemData = (
    WriteStorage<'a, Action>,
    ReadStorage<'a, Player>,
    WriteStorage<'a, Effect>,
    Entities<'a>,
    WriteStorage<'a, Exile>,
    WriteStorage<'a, Inventory>,
    WriteStorage<'a, Item>,
    Read<'a, LazyUpdate>,
    WriteStorage<'a, Looting>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, SuspendPlayer>,
    //Read<'a, UI>,
  );

  fn run(
    &mut self,
    (
      mut _actions,
      players,
      mut _effects,
      entities,
      exiles,
      mut inventories,
      mut items,
      lazy,
      mut lootings,
      positions,
      names,
      mut suspend_controls,
      //ui
    ): Self::SystemData,
  ) {
    // run all looting
    let mut dead_loots = vec![];
    let mut looted_inventory_ents: HashSet<Entity> = HashSet::new();
    for (ent, mut looting) in (&entities, &mut lootings).join() {
      // run the looting
      let (looting_is_done, looting_ents) = self.run_looting(
        &mut looting,
        &mut inventories,
        &entities,
        &lazy,
        &names,
        &positions,
        &players,
        &mut suspend_controls,
        //&ui
      );
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
    for (ent, _inv, _player, _) in
      (&entities, &mut inventories, &players, !&exiles).join()
    {
      let _entity_already_looting_or_being_looted =
        looted_inventory_ents.contains(&ent);
      //if let Some(ctrl) = ui.get_player_controller(player.0) {
      //  // An entity can be looted without wanting to, so they need to be
      //  // able to shut that shit down!
      //  let inv_btn_on =
      //    ctrl.y().is_on_this_frame();
      //  let wants_to_open =
      //    inv_btn_on && !entity_already_looting_or_being_looted;
      //  let wants_to_close =
      //    inv_btn_on && entity_already_looting_or_being_looted;
      //  if wants_to_open {
      //    // Create a looting for it
      //    let _looting =
      //      lazy
      //      .create_entity(&entities)
      //      .with(Looting {
      //        inventory: Some(ent),
      //        looter: ent,
      //        // it's all their own inventory here!
      //        is_looking_in_own_inventory: true,
      //        index: None
      //      })
      //      .build();
      //  } else if wants_to_close {
      //    // Search through all the lootings and find the one
      //    let loot_ent:Entity =
      //      *(&entities, &lootings)
      //      .join()
      //      .filter_map(|(loot_ent, loot)| {
      //        if loot.inventory == Some(ent) || loot.looter == ent {
      //          Some(loot_ent)
      //        } else {
      //          None
      //        }
      //      })
      //      .collect::<Vec<_>>()
      //      .first()
      //      .expect(&format!(
      //        "Player {:?} is trying to cancel a loot that doesn't exist",
      //        names.get(ent)
      //      ));
      //    // Include it to be removed
      //    if !dead_loots.contains(&loot_ent) {
      //      dead_loots
      //        .push(loot_ent);
      //    }
      //  }
      //}
    }

    // destroy all the finished lootings
    dead_loots.into_iter().for_each(|ent| {
      entities.delete(ent).unwrap();
    });

    // run upkeep on all the inventories
    for inventory in (&mut inventories).join() {
      inventory.upkeep(&entities, &mut items, &names);
    }
  }
}
