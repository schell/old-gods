/// The message system allows text to be shown to the user through a creature
/// talking with a word bubble.
///
/// A message/wordbubble is transient. It is created by some other system and
/// shows until a certain time has elapsed and then it is removed from the ECS.
use specs::prelude::*;

use super::super::components::{Exile, Position};
use super::super::systems::screen::Screen;
use super::super::utils::FPSCounter;


pub struct WordBubble {
  _message: String,
  time_left: f32,
}


impl Component for WordBubble {
  type Storage = HashMapStorage<Self>;
}


pub struct MessageSystem;


impl<'a> System<'a> for MessageSystem {
  type SystemData = (
    Entities<'a>,
    ReadStorage<'a, Exile>,
    Read<'a, FPSCounter>,
    Read<'a, LazyUpdate>,
    ReadStorage<'a, Position>,
    Read<'a, Screen>,
    WriteStorage<'a, WordBubble>,
  );

  fn run(
    &mut self,
    (
      entities,
      exiles,
      fps,
      lazy,
      positions,
      screen,
      mut word_bubbles
    ): Self::SystemData,
  ) {
    let elements = (&entities, &positions, &mut word_bubbles, !&exiles).join();
    let area = screen.aabb();
    for (ent, &Position(pos), mut word_bubble, ()) in elements {
      if !area.contains_point(&pos) {
        // this word bubble cannot be seen
        continue;
      }

      word_bubble.time_left -= fps.last_delta();
      if word_bubble.time_left < 0.0 {
        lazy.remove::<WordBubble>(ent);
      }
    }
  }
}
