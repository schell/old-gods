//! Keeps track of any entities that are within the boundaries of a zone.
//!
//! Zones are essentially a cache of entities whose shapes intersect the
//! zone's shape.
use specs::prelude::*;

use super::super::prelude::{AABBTree, Exile, Position, Shape};


/// A Zone is an area that can hold some entities. In order to work properly
/// an entity with a Zone component should also have a Shape component.
#[derive(Debug, Clone)]
pub struct Zone {
    pub inside: Vec<Entity>,
}


impl Component for Zone {
    type Storage = HashMapStorage<Self>;
}


/// The ZoneSystem keeps track of any entities that are within the boundaries of
/// any zone.
/// To be within a zone means that one's shape intersects the zone's shape.
pub struct ZoneSystem;


#[derive(SystemData)]
pub struct ZoneSystemData<'a> {
    aabb_tree: Read<'a, AABBTree>,
    entities: Entities<'a>,
    exiles: ReadStorage<'a, Exile>,
    positions: ReadStorage<'a, Position>,
    shapes: ReadStorage<'a, Shape>,
    zones: WriteStorage<'a, Zone>,
}


impl<'a> System<'a> for ZoneSystem {
    type SystemData = ZoneSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        // Do some generic zone upkeep
        let exiles = &data.exiles;
        for (zone_ent, mut zone, ()) in (&data.entities, &mut data.zones, !exiles).join() {
            let intersections: Vec<Entity> = data
                .aabb_tree
                .query_intersecting_shapes(&data.entities, &zone_ent, &data.shapes, &data.positions)
                .into_iter()
                .filter_map(|(e, _, _)| {
                    if e == zone_ent || exiles.contains(e) {
                        None
                    } else {
                        Some(e)
                    }
                })
                .collect();
            zone.inside = intersections;
        }
    }
}
