use specs::prelude::*;

use super::super::super::components::{Action, Effect, Sprite};


pub struct Door;


impl Door {
  /// Run one door.
  pub fn run(
    actions: &ReadStorage<Action>,
    entities: &Entities,
    ent: Entity,
    lazy: &LazyUpdate,
    sprite: &Sprite,
  ) {
    let children: Vec<&Entity> = sprite.current_children();

    let is_open = sprite.current_keyframe().as_str() == "open";
    let next_keyframe = if is_open { "closed" } else { "open" };

    'find_child: for child in children {
      // In this simplest of doors script, any action is considered
      // a door handle.
      if let Some(action) = actions.get(*child) {
        // See if it has been taken.
        if !action.taken_by.is_empty() {
          // The action procs!
          lazy
            .create_entity(entities)
            .with(Effect::ChangeKeyframe {
              sprite: ent,
              to: next_keyframe.to_string(),
            })
            .build();
          break 'find_child;
        }
      }
    }
  }
}
