use super::super::components::inventory::{Inventory, Item};
/// Manages the looting process.
use old_gods::{
    prelude::{
        Action, Exile, FitnessStrategy, Lifespan, Name, Object, OriginOffset, Player,
        PlayerControllers, Position, Rendering, Shape, AABB, V2,
    },
    utils::clamp,
};
use specs::prelude::*;


/// To facilitate "trade".
pub struct Loot {
    /// The inventory here.
    pub ent_of_inventory_here: Entity,

    /// The inventory there.
    /// A value of 'None' means the looter is looting themselves.
    pub ent_of_inventory_there: Option<Entity>,

    /// Whether or not the looter is looking here.
    pub looking_here: bool,

    /// The index of the item that the looter is currently looking at.
    pub item_index: usize,

    /// Is this loot done?
    pub should_close: bool,
}


impl Loot {
    /// How many columns do we use to display loot?
    pub const COLS: usize = 6;

    pub fn clamp_index(&mut self, items_len: usize) {
        if items_len > 0 {
            let ndx = self.item_index;
            self.item_index = clamp(0, ndx, items_len - 1);
        }
    }

    pub fn pred_index(&mut self, items_len: usize) {
        if items_len > 0 {
            let ndx = self.item_index;
            self.item_index = if ndx > 0 {
                clamp(0, ndx - 1, items_len - 1)
            } else {
                0
            };
        }
    }

    pub fn succ_index(&mut self, items_len: usize) {
        if items_len > 0 {
            let ndx = self.item_index;
            self.item_index = clamp(0, ndx + 1, items_len - 1);
        }
    }
}


pub struct LootingSystem;


#[derive(SystemData)]
pub struct LootingSystemData<'a> {
    entities: Entities<'a>,
    exiles: WriteStorage<'a, Exile>,
    inventories: WriteStorage<'a, Inventory>,
    lazy: Read<'a, LazyUpdate>,
    loots: Write<'a, Vec<Loot>>,
    names: ReadStorage<'a, Name>,
    objects: WriteStorage<'a, Object>,
    offsets: WriteStorage<'a, OriginOffset>,
    positions: ReadStorage<'a, Position>,
    players: ReadStorage<'a, Player>,
    player_controllers: Read<'a, PlayerControllers>,
    renderings: ReadStorage<'a, Rendering>,
    shapes: ReadStorage<'a, Shape>,
}


/// Find actionless items on the map.
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

    lazy.create_entity(&entities)
        .with(a)
        .with(Position(p))
        .with(s)
        .with(Name("pickup item".to_string()))
        .build()
}


#[derive(Clone, PartialEq)]
pub enum LootingResult {
    None,
    Use {
        inv: Entity,
        item_ndx: usize,
    },
    Drop {
        inv: Entity,
        item_ndx: usize,
    },
    Take {
        from: Entity,
        item_ndx: usize,
        to: Entity,
    },
}



fn handle_inventory_action(
    action: LootingResult,
    data: &mut LootingSystemData,
) -> Result<(), String> {
    match action {
        LootingResult::None => {}
        LootingResult::Take { item_ndx, from, to } => {
            let item = data
                .inventories
                .get_mut(from)
                .map(|inv| inv.items.remove(item_ndx))
                .expect("could not get item from index");

            let into_inv = data
                .inventories
                .get_mut(to)
                .expect("could not get inventory to place item into");
            into_inv.add_item(item);
        }
        LootingResult::Drop {
            item_ndx,
            inv: inv_ent,
        } => {
            let inv = data
                .inventories
                .get_mut(inv_ent)
                .expect("could not remove item from inventory");
            let loc = data
                .positions
                .get(inv_ent)
                .map(|p| p.0)
                .expect("tried to drop an item but the dropper has no position");
            let from_aabb = data
                .shapes
                .get(inv_ent)
                .map(|s| s.aabb())
                .unwrap_or(AABB::identity());
            inv.throw_item_with_index_onto_the_map(
                item_ndx,
                loc,
                from_aabb,
                &data.entities,
                &data.lazy,
            );
        }
        LootingResult::Use { .. } => {
            panic!("TODO: use item");
        }
    }

    Ok(())
}


fn run_looting(looting: &mut Loot, data: &mut LootingSystemData) -> Result<(), String> {
    let mut inv_action = LootingResult::None;
    let inventory_here = data
        .inventories
        .get(looting.ent_of_inventory_here)
        .ok_or("inventory here DNE".to_string())?;
    let inventory_there = looting
        .ent_of_inventory_there
        .map(|ent| data.inventories.get(ent))
        .flatten();
    let player = data
        .players
        .get(looting.ent_of_inventory_here)
        .ok_or("TODO: Support looting for npcs.".to_string())?;

    data.player_controllers
        .with_ui_ctrl_at::<_, Result<(), String>>(player.0, |ctrl| {
            let item_len = if looting.looking_here {
                inventory_here.items.len()
            } else {
                inventory_there
                    .ok_or("trying to loot an inventory that DNE".to_string())?
                    .items
                    .len()
            };

            // Determine where the looter is going to put the item - if the looter
            // * is hitting A it means they want to trade the item
            // * is hitting B they want to drop the item onto the map
            // * is hitting X they want to use the item
            if ctrl.right().is_on_this_frame() && looting.looking_here && inventory_there.is_some()
            {
                looting.looking_here = false;
                ctrl.debounce();
            } else if ctrl.left().is_on_this_frame()
                && !looting.looking_here
                && inventory_there.is_some()
            {
                looting.looking_here = true;
                ctrl.debounce();
            } else if ctrl.down().is_on_or_repeated_this_frame() {
                looting.succ_index(item_len);
                ctrl.debounce();
            } else if ctrl.up().is_on_or_repeated_this_frame() {
                looting.pred_index(item_len);
                ctrl.debounce();
            } else if ctrl.y().is_on_this_frame() {
                looting.should_close = true;
                // Switch to the map
                ctrl.use_for_map();
            } else if ctrl.a().is_on_this_frame() {
                // Put this item in the other inventory.
                let from = if looting.looking_here {
                    looting.ent_of_inventory_here
                } else {
                    looting.ent_of_inventory_there.ok_or("inventory DNE")?
                };
                let to = if looting.looking_here {
                    looting.ent_of_inventory_there.ok_or("inventory DNE")?
                } else {
                    looting.ent_of_inventory_here
                };
                inv_action = LootingResult::Take {
                    from,
                    to,
                    item_ndx: looting.item_index,
                };
                ctrl.debounce();
            } else if ctrl.b().is_on_this_frame() {
                // Put this item on the map
                inv_action = LootingResult::Drop {
                    inv: if looting.looking_here {
                        looting.ent_of_inventory_here
                    } else {
                        looting.ent_of_inventory_there.ok_or("inventory DNE")?
                    },
                    item_ndx: looting.item_index,
                };
                ctrl.debounce();
            } else if ctrl.x().is_on_this_frame() {
                // Use this item
                inv_action = LootingResult::Use {
                    inv: if looting.looking_here {
                        looting.ent_of_inventory_here
                    } else {
                        looting.ent_of_inventory_there.ok_or("inventory DNE")?
                    },
                    item_ndx: looting.item_index,
                };
                ctrl.debounce();
            }

            Ok(())
        })
        .unwrap_or(Ok(()))?;

    handle_inventory_action(inv_action, data)?;

    Ok(())
}


/// Initiates new loots.
fn start_new_loots(data: &mut LootingSystemData) {
    // For each player that has an inventory, check to see if they want to loot.
    // If so, add a looting instance for them.
    let joints = (
        &data.entities,
        &mut data.inventories,
        &data.players,
        !&data.exiles,
    )
        .join();

    'finding_looting_requests: for (ent, _inv, player, _) in joints.into_iter() {
        // Continue to the next potential looter if this looter is already
        // looting.
        for loot in data.loots.iter() {
            if loot.ent_of_inventory_here == ent {
                continue 'finding_looting_requests;
            }
        }

        let mut new_loots = vec![];
        data.player_controllers.with_map_ctrl_at(player.0, |ctrl| {
            // An entity can be looted without wanting to, so they need to be
            // able to shut that shit down!
            let wants_to_open = ctrl.y().is_on_this_frame();
            if wants_to_open {
                // Create a looting for it
                new_loots.push(Loot {
                    ent_of_inventory_here: ent,
                    ent_of_inventory_there: Some(ent),
                    // it's all their own inventory here!
                    looking_here: true,
                    item_index: 0,
                    should_close: false,
                });
                // Set the controller to be used for the UI
                ctrl.use_for_ui();
            }
        });
        data.loots.extend(new_loots);
    }
}


impl<'a> System<'a> for LootingSystem {
    type SystemData = LootingSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        // Run lootings
        start_new_loots(&mut data);
        // Get all the lootings
        let mut loots = {
            let loots_ref:&mut Vec<Loot> = &mut data.loots;
            std::mem::replace(loots_ref, vec![])
        };
        // Run all the lootings
        for loot in loots.iter_mut() {
            run_looting(loot, &mut data).unwrap();
        }
        loots.retain(|loot| !loot.should_close);
        // Put the remaining lootings back
        *data.loots = loots;
    }
}
