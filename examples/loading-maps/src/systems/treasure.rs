use specs::prelude::{
  Entity,
  Entities,
  LazyUpdate,
  System,
  ReadStorage,
};

use old_gods::components::{
  Action, Effect, Inventory, Name, Sprite,
};


pub struct TreasureChestSystem;


impl TreasureChestSystem {
  /// Get the treasure's inventory
  pub fn inventory(
    children: &Vec<Entity>,
    inventories: &ReadStorage<Inventory>,
  ) -> Entity {
    for entity in children {
      if inventories.contains(*entity) {
        return entity.clone();
      }
    }
    panic!("Could not find a container's inventory")
  }
}


//impl<'s> System<'s> for TreasureChestSystem {
//  type SystemData = (
//    ReadStorage<'s, Action>,
//    Entities<'s>,
//    ReadStorage<'s, Inventory>,
//    LazyUpdate<'s>,
//    ReadSTorage<'s, Name>
//  );
//
//  /// Run one container.
//  pub fn run(
//    actions: &ReadStorage<Action>,
//    entities: &Entities,
//    ent: Entity,
//    inventories: &ReadStorage<Inventory>,
//    lazy: &LazyUpdate,
//    names: &ReadStorage<Name>,
//    sprite: &Sprite,
//  ) {
//    let children: Vec<Entity> = sprite
//      .current_children()
//      .into_iter()
//      .map(|c| c.clone())
//      .collect();
//
//    for child in &children {
//      if let Some(action) = actions.get(*child) {
//        let name = names.get(*child).expect("A sprite action has no name!");
//        // See if it has been taken.
//        if !action.taken_by.is_empty() {
//          // The action procs!
//          match name.0.as_str() {
//            "open" => {
//              println!("Opening container {:?}", sprite.keyframe);
//              lazy
//                .create_entity(entities)
//                .with(Effect::ChangeKeyframe {
//                  sprite: ent,
//                  to: "open".to_string(),
//                })
//                .build();
//            }
//            "close" => {
//              println!("Closing container {:?}", sprite.keyframe);
//              lazy
//                .create_entity(entities)
//                .with(Effect::ChangeKeyframe {
//                  sprite: ent,
//                  to: "close".to_string(),
//                })
//                .build();
//            }
//            "loot" => {
//              let inv: Entity = Container::inventory(&children, inventories);
//              for looter in &action.taken_by {
//                lazy
//                  .create_entity(entities)
//                  .with(Effect::LootInventory {
//                    inventory: Some(inv),
//                    looter: *looter,
//                  })
//                  .build();
//              }
//              //// Later, exile the action so it doesn't show during the loot process.
//              //Exile::exile_later(*child, ExiledBy("container"), &updater);
//            }
//            s => {
//              panic!("Unsupported container action named {:?}", s);
//            }
//          }
//        }
//      }
//    }
//  }
//}
