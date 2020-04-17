use super::super::components::{Action, FitnessStrategy, Inventory};
use log::trace;
use old_gods::{
    parser::*,
    prelude::{
        Entities, Entity, Exile, Join, LazyUpdate, Name, Player, PlayerControllers, Read,
        ReadStorage, System, WriteStorage, Zone,
    },
};

#[derive(Debug, PartialEq)]
pub enum FitnessResult {
    Fit,
    UnfitDoesntHaveItem,
    UnfitDoesntHaveInventory,
    Unfit,
}


/// Determine whether or not the target entity is fit to take this action.
fn target_is_fit<'a>(
    strategy: &FitnessStrategy,
    target_entity: &Entity,
    inventories: &ReadStorage<'a, Inventory>,
    names: &ReadStorage<'a, Name>,
) -> FitnessResult {
    match strategy {
        FitnessStrategy::HasItem(name) => {
            println!("  looking for item {:?}", name);
            let has_item = inventories
                .get(*target_entity)
                .map(|inv| {
                    for item in inv.item_iter() {
                        println!("  checking item {:?}", item);
                        if name == &item.name {
                            return true;
                        }
                    }
                    false
                })
                .unwrap_or(false);
            if has_item {
                FitnessResult::Fit
            } else {
                FitnessResult::UnfitDoesntHaveItem
            }
        }

        FitnessStrategy::HasInventory => {
            let has_inventory = inventories.contains(*target_entity);
            if has_inventory {
                FitnessResult::Fit
            } else {
                FitnessResult::UnfitDoesntHaveInventory
            }
        }

        FitnessStrategy::All(strategies) => {
            for strategy in strategies {
                let fitness = target_is_fit(&strategy, target_entity, inventories, names);
                if fitness != FitnessResult::Fit {
                    return fitness;
                }
            }
            FitnessResult::Fit
        }

        FitnessStrategy::Any(strategies) => {
            for strategy in strategies {
                let fitness = target_is_fit(&strategy, target_entity, inventories, names);
                if fitness == FitnessResult::Fit {
                    return fitness;
                }
            }
            FitnessResult::Unfit
        }
    }
}


/// ## The player system/step
pub struct ActionSystem;


impl<'a> System<'a> for ActionSystem {
    type SystemData = (
        WriteStorage<'a, Action>,
        Entities<'a>,
        ReadStorage<'a, Exile>,
        ReadStorage<'a, Inventory>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Player>,
        Read<'a, PlayerControllers>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Zone>,
    );

    fn run(
        &mut self,
        (mut actions, entities, exiles, inventories, lazy, players, gamepads, names, mut zones): Self::SystemData,
    ) {
        // Find any actions that don't have zones, then create zones for them.
        // A zone will keep track of any entities intersecting the action.
        (&entities, &actions, !&zones)
            .join()
            .map(|(e, _, ())| e.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|ent| {
            let _ = zones.insert(ent, Zone { inside: vec![] });
        });

        // Run through each action and test the fitness of any entities in its zone
        for (action_ent, mut action, zone, ()) in (&entities, &mut actions, &zones, !&exiles).join()
        {
            // Reset the action's coffers
            action.taken_by = vec![];
            action.elligibles = vec![];

            'neighbors: for inside_ent in &zone.inside {
                let inside_ent = *inside_ent;
                // Determine the fitness of the toon for this action
                let fitness = target_is_fit(&action.strategy, &inside_ent, &inventories, &names);
                if fitness != FitnessResult::Fit {
                    continue;
                }
                trace!(
                    "{:?} is fit for {:?}",
                    names.get(inside_ent),
                    names.get(action_ent)
                );
                action.elligibles.push(inside_ent);

                if let Some(player) = players.get(inside_ent) {
                    let action_is_dead =
                        gamepads.with_map_ctrl_at(player.0, |ctrl| {
                            if ctrl.a().is_on_this_frame() {
                                // Allow some other system to handle it.
                                action.taken_by.push(inside_ent);
                                // Decrement the actions life counter, if it's dead we'll cull it next
                                // frame.
                                action.lifespan = action.lifespan.pred();
                                // Show some stuff for debugging
                                trace!(
                                    "Action {:?} was taken by {:?} and has {:?} uses remaining.",
                                    action.text,
                                    names.get(inside_ent),
                                    action.lifespan
                                );
                                if action.lifespan.is_dead() {
                                    trace!("  this action is dead. Removing lazily.");
                                    lazy.remove::<Action>(action_ent);
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false);
                    // If the action is now dead, don't let any other neighbors take it
                    // and lazily remove it.
                    if action_is_dead {
                        break 'neighbors;
                    }
                }
            }
        }
    }
}
