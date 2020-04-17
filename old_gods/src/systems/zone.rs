//! Keeps track of any entities that are within the boundaries of a zone.
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


impl<'a> System<'a> for ZoneSystem {
    type SystemData = (
        Read<'a, AABBTree>,
        Entities<'a>,
        ReadStorage<'a, Exile>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Shape>,
        WriteStorage<'a, Zone>,
    );

    fn run(
        &mut self,
        (aabb_tree, entities, exiles, positions, shapes, mut zones): Self::SystemData,
    ) {
        // Do some generic zone upkeep
        for (zone_ent, mut zone, ()) in (&entities, &mut zones, !&exiles).join() {
            let intersections: Vec<Entity> = aabb_tree
                .query_intersecting_shapes(&entities, &zone_ent, &shapes, &positions)
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
