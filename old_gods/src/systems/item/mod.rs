/// Controls picking up items from the map and placing them in inventories.
use specs::prelude::*;

use super::super::prelude::{
  Action, Effect, Exile, FitnessStrategy, Inventory, Lifespan, Name, Position,
  Shape, Sprite, V2,
};


pub struct ItemSystem;


impl ItemSystem {
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
}


impl<'a> System<'a> for ItemSystem {
  type SystemData = (
    Entities<'a>,
    WriteStorage<'a, Action>,
    WriteStorage<'a, Exile>,
    ReadStorage<'a, Item>,
    WriteStorage<'a, Inventory>,
    Read<'a, LazyUpdate>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, Shape>,
    ReadStorage<'a, Sprite>,
  );
  fn run(
    &mut self,
    (
      entities,
      actions,
      exiles,
      items,
      inventories,
      lazy,
      names,
      mut positions,
      shapes,
      sprites,
    ): Self::SystemData,
  ) {
    for (ent, Item { .. }, name, ()) in
      (&entities, &items, &names, !&exiles).join()
    {
      // Determine if this is a map item or an inventory item
      let may_pos = positions.get(ent).cloned();
      let item_on_map = may_pos.is_some();
      if item_on_map {
        let position = may_pos.unwrap();
        // Determine if this item has a pickup action (we'll store it using a sprite)
        if let Some(sprite) = sprites.get(ent) {
          let action_ent = sprite
            .top_level_children
            .first()
            .cloned()
            .expect("Item sprite doesn't have any entities");
          let action = actions
            .get(action_ent)
            .expect("Item sprite doesn't have a pickup action component");
          // Give the action to the first elligible taker.
          'action_taken: for taker in &action.taken_by {
            let taker_has_inventory = inventories.contains(*taker);
            if taker_has_inventory {
              // It has been taken, so put a pickup effect in the ECS.
              let pickup_effect = Effect::InsertItem {
                item: ent,
                into: Some(*taker),
                from: None,
              };
              lazy.create_entity(&entities).with(pickup_effect).build();
              // Delete the pickup action and sprite
              entities.delete(action_ent).unwrap();
              lazy.remove::<Sprite>(ent);
              break 'action_taken;
            }
          }

          // Make sure the action position stays up to date with the item
          positions
            .insert(action_ent, position)
            .expect("Could not insert item pickup action position");
        } else {
          // It does not!
          // Create a sprite component and a pickup action for it.
          let action_ent = self.new_pickup_action(
            &entities,
            &lazy,
            name.0.clone(),
            position.0,
            shapes.get(ent),
          );
          let sprite = Sprite::with_top_level_children(vec![action_ent]);
          lazy.insert(ent, sprite);
        }
      } else {
        // This is an inventory item and there's nothing to do
      }
    }
  }
}
