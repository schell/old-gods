use super::super::components::{Action, FitnessStrategy, Inventory, Lifespan};
use log::trace;
use old_gods::prelude::{
    Entities, Entity, Exile, Join, LazyUpdate, Name, Object, Player, PlayerControllers, Read,
    ReadStorage, System, WriteStorage, Zone,
};
use serde_json::Value;
use specs::prelude::*;


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


#[derive(SystemData)]
pub struct ActionSystemData<'a> {
    actions: WriteStorage<'a, Action>,
    entities: Entities<'a>,
    exiles: ReadStorage<'a, Exile>,
    inventories: ReadStorage<'a, Inventory>,
    lazy: Read<'a, LazyUpdate>,
    objects: WriteStorage<'a, Object>,
    players: ReadStorage<'a, Player>,
    gamepads: Read<'a, PlayerControllers>,
    names: ReadStorage<'a, Name>,
    zones: WriteStorage<'a, Zone>,
}


impl<'a> System<'a> for ActionSystem {
    type SystemData = ActionSystemData<'a>;

    fn run(&mut self, mut data: ActionSystemData) {
        // Find objects that have action types and turn them into actions, deleting the object.
        let mut remove_objects = vec![];
        for (ent, obj) in (&data.entities, &mut data.objects).join() {
            if &obj.type_is != "action" {
                continue;
            }
            remove_objects.push(ent);

            let properties = obj.json_properties();
            let text_value: &Value = properties
                .get("text")
                .expect("An action must have a 'text' property");
            let text: String = text_value
                .as_str()
                .expect("An action's 'text' property must be a string")
                .to_string();
            let strategy = properties
                .get("strategy")
                .expect("An action must have a 'fitness' property")
                .as_str()
                .map(|s| {
                    FitnessStrategy::try_from_str(s)
                        .map_err(|e| format!("Could not parse action's fitness strategy: {:?}", e))
                        .unwrap()
                })
                .expect("An action's 'fitness' property must be a string");
            let lifespan_val: &Value = properties
                .get("lifespan")
                .expect("An action must have a 'lifespan' property");

            let lifespan = if Some("forever") == lifespan_val.as_str() {
                Lifespan::Forever
            } else if let Some(num) = lifespan_val.as_u64() {
                Lifespan::Many(num as u32)
            } else {
                panic!(
                    "lifespan value must be the string \"forever\" or an int. Found '{}'",
                    lifespan_val
                )
            };

            let action = Action {
                elligibles: vec![],
                taken_by: vec![],
                text,
                strategy,
                lifespan,
            };
            let _ = data.actions.insert(ent, action);
        }

        for ent in remove_objects.into_iter() {
            let _ = data.objects.remove(ent);
        }

        // Find any actions that don't have zones, then create zones for them.
        // A zone will keep track of any entities intersecting the action.
        (&data.entities, &data.actions, !&data.zones)
            .join()
            .map(|j| j.0)
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|ent| {
                let _ = data.zones.insert(ent, Zone { inside: vec![] });
            });

        // Run through each action and test the fitness of any entities in its zone
        for (action_ent, mut action, zone, ()) in (
            &data.entities,
            &mut data.actions,
            &data.zones,
            !&data.exiles,
        )
            .join()
        {
            // Reset the action's coffers
            action.taken_by = vec![];
            action.elligibles = vec![];

            'neighbors: for inside_ent in &zone.inside {
                let inside_ent = *inside_ent;
                // Determine the fitness of the toon for this action
                let fitness = target_is_fit(
                    &action.strategy,
                    &inside_ent,
                    &data.inventories,
                    &data.names,
                );
                if fitness != FitnessResult::Fit {
                    continue;
                }
                trace!(
                    "{:?} is fit for {:?}",
                    data.names.get(inside_ent),
                    data.names.get(action_ent)
                );
                action.elligibles.push(inside_ent);

                let names = &data.names;
                let lazy = &data.lazy;

                if let Some(player) = data.players.get(inside_ent) {
                    let action_is_dead = data
                        .gamepads
                        .with_map_ctrl_at(player.0, |ctrl| {
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
