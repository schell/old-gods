/// Manages the creation of inventories and items from tiled Object and JSON components.
use super::super::components::{Inventory, Item};
use log::trace;
use old_gods::prelude::{
    AABBTree, Barrier, Name, Object, OriginOffset, Position, Rendering, Shape, JSON,
};
use specs::prelude::*;
use std::collections::HashMap;


/// Used to associate an entity with an inventory that may be found later.
#[derive(Debug)]
struct InventoryClaim {
    entity: Entity,
    inventory_name: String,
}


/// Used to associate an inventory with items and an inventory holder taht may be
/// found later.
struct UnclaimedInventory {
    entity: Entity,
    inventory: Inventory,
}


pub struct InventorySystem {
    inventory_claims: Vec<InventoryClaim>,
    new_unclaimed_inventories: HashMap<String, UnclaimedInventory>,
    unclaimed_inventories: HashMap<String, UnclaimedInventory>,
}


impl InventorySystem {
    pub fn new() -> Self {
        InventorySystem {
            inventory_claims: vec![],
            new_unclaimed_inventories: HashMap::new(),
            unclaimed_inventories: HashMap::new(),
        }
    }

    /// Associate unclaimed inventories with inventory claims, if possible.
    fn resolve_claims(&mut self, data: &mut InventorySystemData) {
        let mut claims = std::mem::replace(&mut self.inventory_claims, vec![]);

        claims.retain(|claim| {
            if let Some(unclaimed) = self.unclaimed_inventories.remove(&claim.inventory_name) {
                trace!(
                    "resolved inventory '{}' with {} items.",
                    claim.inventory_name,
                    unclaimed.inventory.item_len()
                );
                data.entities.delete(unclaimed.entity).expect("delete ent");
                data.inventories
                    .insert(claim.entity, unclaimed.inventory)
                    .expect("insert inv");
                false
            } else {
                true
            }
        });

        self.inventory_claims = claims;
    }
}


#[derive(SystemData)]
pub struct InventorySystemData<'a> {
    aabb_tree: Write<'a, AABBTree>,
    barriers: ReadStorage<'a, Barrier>,
    entities: Entities<'a>,
    inventories: WriteStorage<'a, Inventory>,
    items: WriteStorage<'a, Item>,
    jsons: WriteStorage<'a, JSON>,
    names: ReadStorage<'a, Name>,
    objects: WriteStorage<'a, Object>,
    offsets: WriteStorage<'a, OriginOffset>,
    positions: ReadStorage<'a, Position>,
    renderings: ReadStorage<'a, Rendering>,
    shapes: ReadStorage<'a, Shape>,
}


/// Find any objects with inventory or item types that don't already have inventory components
/// so we can create them.
/// Delete the object component afterward, if found.
fn create_new_inventories(
    data: &mut InventorySystemData,
) -> Result<HashMap<String, UnclaimedInventory>, String> {
    let mut invs = HashMap::new();
    let mut removes = vec![];
    let mut num_items = 0;
    let mut num_invs = 0;
    for (ent, obj, _, ()) in (
        &data.entities,
        &data.objects,
        &data.shapes,
        !&data.inventories,
    )
        .join()
    {
        match obj.type_is.as_str() {
            "inventory" => {
                num_invs += 1;
                removes.push(ent);

                if obj.name.is_empty() {
                    return Err("inventory must have a name property".to_string());
                }
                // We have to have the items to put into the inv first, so we just store
                // this to process it later.
                invs.insert(
                    obj.name.clone(),
                    UnclaimedInventory {
                        entity: ent,
                        inventory: Inventory::new(vec![]),
                    },
                );

                trace!(
                    "{:#?} pos: {:#?} shape: {:#?}",
                    data.names.get(ent),
                    data.positions.get(ent),
                    data.shapes.get(ent)
                );
            }

            "item" => {
                num_items += 1;
                removes.push(ent);

                let properties = obj.json_properties();
                let rendering = data
                    .renderings
                    .get(ent)
                    .ok_or_else(|| "item has no rendering".to_string())?;
                let shape = data
                    .shapes
                    .get(ent)
                    .cloned()
                    .ok_or_else(|| "item must have a shape")?;
                trace!(
                    "{:#?} pos: {:#?} shape: {:#?}",
                    data.names.get(ent),
                    data.positions.get(ent),
                    data.shapes.get(ent)
                );
                let offset = data.offsets.get(ent).cloned();
                let is_barrier = data.barriers.contains(ent);
                let item = Item {
                    name: obj.name.clone(),
                    usable: properties
                        .get("usable")
                        .map(|v| v.as_bool())
                        .flatten()
                        .unwrap_or(false),
                    stack: properties
                        .get("stack_count")
                        .map(|v| v.as_u64().map(|u| u as usize))
                        .flatten(),
                    rendering: rendering.clone(),
                    shape,
                    offset,
                    is_barrier,
                };

                let _ = data.items.insert(ent, item);
            }

            _ => {}
        }
    }
    // Remove the item's objects
    removes.into_iter().for_each(|ent| {
        let _ = data.objects.remove(ent);
    });

    if num_invs + num_items > 0 {
        trace!("created {} invs and {} items", num_invs, num_items);
    }

    Ok(invs)
}


/// Use the entities' shapes to locate any items on the map that belong in the
/// newly created inventories and add them.
fn fill_inventories(
    invs: &mut HashMap<String, UnclaimedInventory>,
    data: &mut InventorySystemData,
) -> Result<(), String> {
    for (_, unclaimed_inventory) in invs.iter_mut() {
        // The inventory should already have a shape from the TiledSystem,
        // so we can use it to query for any items that may be intersecting, and
        // then place those in the inventory.
        let pos = data
            .positions
            .get(unclaimed_inventory.entity)
            .ok_or("no position")?;
        let shape = data
            .shapes
            .get(unclaimed_inventory.entity)
            .ok_or("no shape")?;
        let items: Vec<Item> = data
            .aabb_tree
            .query(
                &data.entities,
                &shape.aabb().translate(&pos.0),
                &unclaimed_inventory.entity,
            )
            .into_iter()
            .filter_map(|(ent, _)| {
                // Take the item off the map in preparation to place it in the inventory
                if let Some(item) = data.items.remove(ent) {
                    data.entities
                        .delete(ent)
                        .expect("could not delete inventory item entity");
                    Some(item)
                } else {
                    None
                }
            })
            .collect();
        let _ = unclaimed_inventory.inventory.replace_items(items);
    }
    Ok(())
}


/// Find any claims for inventories.
fn find_new_inventory_claims(data: &mut InventorySystemData) -> Vec<InventoryClaim> {
    let mut claims = vec![];
    for (holder_ent, JSON(properties), ()) in
        (&data.entities, &mut data.jsons, !&data.inventories).join()
    {
        if let Some(name) = properties
            .remove("inventory_name")
            .map(|v| v.as_str().map(|s| s.to_string()))
            .flatten()
        {
            claims.push(InventoryClaim {
                entity: holder_ent,
                inventory_name: name,
            });
        }
    }
    claims
}

impl<'a> System<'a> for InventorySystem {
    type SystemData = InventorySystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        // Move the inventories we got last frame
        let past_invs = std::mem::replace(&mut self.new_unclaimed_inventories, HashMap::new());
        self.unclaimed_inventories.extend(past_invs);

        // Create new inventories, store them for a frame to allow some items to appear on the map
        self.new_unclaimed_inventories = create_new_inventories(&mut data).unwrap();

        // Fill the unclaimed inventories
        fill_inventories(&mut self.unclaimed_inventories, &mut data).unwrap();
        let claims = find_new_inventory_claims(&mut data);
        self.inventory_claims.extend(claims);
        self.resolve_claims(&mut data);
    }
}
