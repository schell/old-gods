/// Manages:
/// * moving players based on their controllers' axes
/// * adding and removing take-action components, allowing the ActionSystem to do
///   its job
use specs::prelude::*;

use super::super::components::{Exile, TakeAction, Velocity};
//use super::super::geom::V2;
//use super::ui::UI;

mod record;
pub use self::record::*;


/// A component for suspending control of an entity without exiling it.
pub struct SuspendPlayer;


impl Component for SuspendPlayer {
  type Storage = HashMapStorage<SuspendPlayer>;
}


/// Players the movement and actions taken by characters.
pub struct PlayerSystem;


/// The PlayerSystem carries out motivations on behalf of toons.
impl<'a> System<'a> for PlayerSystem {
  type SystemData = (
    Entities<'a>,
    //Read<'a, UI>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Exile>,
    ReadStorage<'a, MaxSpeed>,
    ReadStorage<'a, SuspendPlayer>,
    WriteStorage<'a, TakeAction>,
    WriteStorage<'a, Velocity>,
  );

  fn run(
    &mut self,
    (entities,
     //ui,
     players,
     exiles,
     max_speeds,
     suspensions,
     mut take_actions,
     mut velocities,
    ): Self::SystemData
  ) {
    // Run over all players and enforce their motivations.
    for (ent, player, _) in (&entities, &players, !&exiles).join() {
      // Remove any previous TakeAction from this toon to begin with
      take_actions.remove(ent);

      let _v =
        velocities
        .get_mut(ent)
        .expect(&format!("Player {:?} does not have velocity.", player));

      let _max_speed:MaxSpeed =
        max_speeds
        .get(ent)
        .map(|mv| mv.clone())
        .unwrap_or(MaxSpeed(100.0));

      // If this toon's control is suspended (taken by the UI, etc)
      // then abort
      if let Some(_) = suspensions.get(ent) {
        continue;
      }

      //// Get the player's controller
      //if let Some(ctrl) = ui.get_player_controller(player.0) {
      //  // Update the velocity of the toon based on the
      //  // player's controller
      //  let ana = ctrl.analog_rate();
      //  let rate =
      //    ana
      //    .unitize()
      //    .unwrap_or(V2::new(0.0, 0.0));
      //  let mult = rate.scalar_mul(max_speed.0);
      //  v.0 = mult;

      //  // Add a TakeAction if the player has hit the A button
      //  if ctrl.a().is_on_this_frame() {
      //    take_actions
      //      .insert(ent, TakeAction)
      //      .expect("Could not insert TakeAction.");
      //  }
      //}
    }
  }
}
