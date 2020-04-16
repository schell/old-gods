use super::super::components::inventory::{Inventory, Item};
/// Manages the looting process.
use old_gods::{
    prelude::{
        Action, Barrier, Easing, Exile, FitnessStrategy, Lifespan, Name, Object, OriginOffset,
        Player, PlayerController, PlayerControllers, Position, Rendering, Shape, Tween, TweenParam,
        Velocity, AABB, V2, ZLevel
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

    /// The x index of the item that the looter is currently looking at.
    pub cursor_x: i32,

    /// The y index of the item that the looter is currently looking at.
    pub cursor_y: i32,

    /// Is this loot done?
    pub should_close: bool,
}


impl Loot {
    /// How many columns do we use to display loot?
    pub const COLS: usize = 6;
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
    z_levels: ReadStorage<'a, ZLevel>
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
        item_ndx: (usize, usize),
    },
    Drop {
        inv: Entity,
        item_ndx: (usize, usize),
    },
    Take {
        from: Entity,
        item_ndx: (usize, usize),
        to: Entity,
    },
}


/// Finds a position around the holder that's out of the way, and then throws the item there.
pub fn throw_item_with_index_onto_the_map(
    inventory: &mut Inventory,
    (x, y): (usize, usize),
    starting_loc: V2,
    from_aabb: AABB,
    z: ZLevel,
    entities: &Entities,
    lazy: &LazyUpdate,
) -> Result<(), String> {
    let item = inventory.remove_xy(x, y).ok_or("no item at x y")?;

    let item_aabb = item.shape.aabb();
    // From there we must offset it some amount to account for
    // the barriers of each
    let radius = {
        let f = from_aabb.greater_extent();
        let i = item_aabb.greater_extent();
        f32::max(f, i)
    };

    // Find a place for the item
    let radians = inventory.dequeue_ejection_in_radians();
    let dv = V2::new(f32::cos(radians), f32::sin(radians));
    let loc = starting_loc + (dv.scalar_mul(radius));

    // Throw the item!
    let speed = 100.0;
    let starting_v = dv.scalar_mul(speed);
    let ent = lazy
        .create_entity(entities)
        .with(Name(item.name.clone()))
        .with(Position(loc))
        .with(Velocity(starting_v))
        .with(item.rendering.clone())
        .with(item.shape.clone())
        .with(item.clone())
        .with(z)
        .build();
    if let Some(offset) = item.offset {
        lazy.insert(ent, offset.clone());
    }
    if item.is_barrier {
        lazy.insert(ent, Barrier);
    }

    // Tween the item flying out of the inventory, eventually stopping.
    let _ = lazy
        .create_entity(entities)
        .with(Tween::new(
            ent,
            TweenParam::Velocity(starting_v, V2::origin()),
            Easing::Linear,
            0.5,
        ))
        .build();

    println!("dv:{:?} radius:{:?} vel:{:?}", dv, radius, starting_v);

    Ok(())
}


fn handle_inventory_action(
    action: LootingResult,
    data: &mut LootingSystemData,
) -> Result<(), String> {
    match action {
        LootingResult::None => {}
        LootingResult::Take {
            item_ndx: (x, y),
            from,
            to,
        } => {
            let item = data
                .inventories
                .get_mut(from)
                .ok_or("could not get inventory to get item from")?
                .remove_xy(x, y)
                .ok_or("could not get item from inventory")?;

            let into_inv = data
                .inventories
                .get_mut(to)
                .ok_or("could not get inventory to place item into")?;
            into_inv.add_item(item);
        }
        LootingResult::Drop {
            item_ndx,
            inv: inv_ent,
        } => {
            let inv = data
                .inventories
                .get_mut(inv_ent)
                .ok_or("could not remove item from inventory")?;
            let loc = data
                .positions
                .get(inv_ent)
                .map(|p| p.0)
                .ok_or("tried to drop an item but the dropper has no position")?;
            let from_aabb = data
                .shapes
                .get(inv_ent)
                .map(|s| s.aabb())
                .unwrap_or(AABB::identity());
            let z = data
                .z_levels
                .get(inv_ent)
                .cloned()
                .ok_or("cannot drop item from an inventory w/o z")?;
            throw_item_with_index_onto_the_map(
                inv,
                item_ndx,
                loc,
                from_aabb,
                z,
                &data.entities,
                &data.lazy,
            )?;
        }
        LootingResult::Use { .. } => {
            panic!("TODO: use item");
        }
    }

    Ok(())
}


pub fn clamp_cursors(cursor_x: &mut i32, cursor_y: &mut i32, items_len: usize) {
    let x = *cursor_x;
    *cursor_x = clamp(0, x, Loot::COLS as i32 - 1);

    if items_len > 0 {
        let rows = (items_len as f32 / Loot::COLS as f32).ceil() as i32;
        let y = *cursor_y;
        *cursor_y = clamp(0, y, rows - 1);
    } else {
        *cursor_y = 0;
    }
}


/// Determine where the looter wants to put the cursor.
fn browse_inventory(
    ctrl: &PlayerController,
    cursor_x: &mut i32,
    cursor_y: &mut i32,
    items_len: usize,
) {
    if ctrl.right().is_on_or_repeated_this_frame() {
        *cursor_x += 1;
    }
    if ctrl.left().is_on_or_repeated_this_frame() {
        *cursor_x -= 1;
    }
    if ctrl.down().is_on_or_repeated_this_frame() {
        *cursor_y += 1;
    }
    if ctrl.up().is_on_or_repeated_this_frame() {
        *cursor_y -= 1;
    }
    clamp_cursors(cursor_x, cursor_y, items_len);
}

/// Determine where the looter is going to put an item.
/// If the looter:
/// * is hitting A it means they want to trade the item
/// * is hitting B they want to drop the item onto the map
/// * is hitting X they want to use the item
fn determine_action(ctrl: &PlayerController, looting: &Loot) -> Result<LootingResult, String> {
    let res = if ctrl.a().is_on_this_frame() {
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

        ctrl.debounce();

        LootingResult::Take {
            from,
            to,
            item_ndx: (looting.cursor_x as usize, looting.cursor_y as usize),
        }
    } else if ctrl.b().is_on_this_frame() {
        ctrl.debounce();
        // Put this item on the map
        LootingResult::Drop {
            inv: if looting.looking_here {
                looting.ent_of_inventory_here
            } else {
                looting.ent_of_inventory_there.ok_or("inventory DNE")?
            },
            item_ndx: (looting.cursor_x as usize, looting.cursor_y as usize),
        }
    } else if ctrl.x().is_on_this_frame() {
        ctrl.debounce();
        // Use this item
        LootingResult::Use {
            inv: if looting.looking_here {
                looting.ent_of_inventory_here
            } else {
                looting.ent_of_inventory_there.ok_or("inventory DNE")?
            },
            item_ndx: (looting.cursor_x as usize, looting.cursor_y as usize),
        }
    } else {
        LootingResult::None
    };
    Ok(res)
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
            browse_inventory(
                ctrl,
                &mut looting.cursor_x,
                &mut looting.cursor_y,
                if looting.looking_here {
                    inventory_here.item_len()
                } else {
                    inventory_there.ok_or("inventory there DNE")?.item_len()
                },
            );

            let may_item_at_cursor = if looting.looking_here {
                inventory_here.item_at_xy(looting.cursor_x, looting.cursor_y)
            } else {
                inventory_there
                    .ok_or("inventory there DNE for select")?
                    .item_at_xy(looting.cursor_x, looting.cursor_y)
            };

            // Close the inventory if the player hits y
            if ctrl.y().is_on_this_frame() {
                looting.should_close = true;
                // Switch to the map
                ctrl.use_for_map();
            } else if may_item_at_cursor.is_some() {
                inv_action = determine_action(ctrl, looting)?;
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
                    cursor_x: 0,
                    cursor_y: 0,
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
            let loots_ref: &mut Vec<Loot> = &mut data.loots;
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
