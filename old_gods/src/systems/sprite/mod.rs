use specs::prelude::*;

use std::collections::HashSet;

mod record;
//mod warp;

pub use self::record::*;
//pub use self::warp::*;


/// The sprite system controls exiling and domesticating other entities based on
/// an entity's Sprite component's keyframe.
pub struct SpriteSystem;


impl<'a> System<'a> for SpriteSystem {
  type SystemData = (
    Entities<'a>,
    WriteStorage<'a, Exile>,
    WriteStorage<'a, Sprite>,
  );

  fn run(&mut self, (entities, mut exiles, mut sprites): Self::SystemData) {
    for (ent, sprite) in (&entities, &mut sprites).join() {
      let should_skip =
      // If this sprite is exiled, skip it
      exiles.contains(ent)
        // If this sprite does not need its keyframe switched, skip it.
        || sprite.keyframe.is_none();
      if should_skip {
        continue;
      }
      let keyframe = sprite.keyframe.take().unwrap();
      // Switch the keyframe of the sprite
      sprite.switch_keyframe(&keyframe, &mut exiles);
    }
  }
}
