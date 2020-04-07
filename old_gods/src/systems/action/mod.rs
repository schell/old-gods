use specs::prelude::*;

use super::super::components::{Exile, Inventory, Name, Zone};
use super::super::parser::*;
use log::trace;


/// ## The player system/step
pub struct ActionSystem;


impl<'a> System<'a> for ActionSystem {
  type SystemData = (
    WriteStorage<'a, Action>,
    Entities<'a>,
    ReadStorage<'a, Exile>,
    ReadStorage<'a, Inventory>,
    Read<'a, LazyUpdate>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, TakeAction>,
    ReadStorage<'a, Zone>,
  );

  fn run(
    &mut self,
    (
      mut actions,
      entities,
      exiles,
      inventories,
      lazy,
      names,
      mut take_actions,
      zones,
    ): Self::SystemData,
  ) {
    // Find any actions that don't have zones, then create zones for them.
    // A zone will keep track of any entities intersecting the action.
    for (ent, _, ()) in (&entities, &actions, !&zones).join() {
      lazy.insert(ent, Zone { inside: vec![] });
    }

    // Run through each action and test the fitness of any entities in its zone
    for (action_ent, mut action, zone, ()) in
      (&entities, &mut actions, &zones, !&exiles).join()
    {
      // Reset the action's coffers
      action.taken_by = vec![];
      action.elligibles = vec![];

      'neighbors: for inside_ent in &zone.inside {
        let inside_ent = *inside_ent;
        // Determine the fitness of the toon for this action
        let fitness =
          action
            .strategy
            .target_is_fit(&inside_ent, &inventories, &names);
        if fitness != FitnessResult::Fit {
          continue;
        }
        // Display the action to the player in the UI.
        trace!(
          "{:?} is fit for {:?}",
          names.get(inside_ent),
          names.get(action_ent)
        );
        action.elligibles.push(inside_ent);

        // Is this elligible player already taking an action?
        // NOTE: The TakeAction component is maintained by the PlayerSystem
        let is_taking_action = take_actions.get(inside_ent).is_some();
        if is_taking_action {
          // Eat that take action
          take_actions.remove(inside_ent);
          // Allow some other system to handle it.
          action.taken_by.push(inside_ent);
          // Decrement the actions life counter, if it's dead we'll cull it next
          // frame.
          action.lifespan = action.lifespan.pred();
          // Show some stuff for debugging
          println!(
            "Action {:?} was taken by {:?} and has {:?} uses remaining.",
            action.text,
            names.get(inside_ent),
            action.lifespan
          );
          // If the action is now dead, don't let any other neighbors take it
          // and lazily remove it.
          if action.lifespan.is_dead() {
            println!("  this action is dead. Removing lazily.");
            lazy.remove::<Action>(action_ent);
            break 'neighbors;
          }
        }
      }
    }
  }
}
