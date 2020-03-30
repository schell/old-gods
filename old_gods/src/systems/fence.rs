/// A fence is a line of demarcation.
/// The fence system tracks what entities have crossed a fence.
use specs::prelude::*;

use std::collections::HashMap;

use super::super::components::{Position, Velocity, ZLevel};
use super::super::geom::{AABBTree, EntityBounds, LineSegment, AABB, V2};


/// A fence is used to track entities that cross it, and at what angle.
#[derive(Debug, Clone)]
pub struct Fence {
  /// The points in this fence.
  pub points: Vec<V2>,

  /// The entities being watched and their last known positions.
  pub watching: HashMap<Entity, V2>,

  /// the entities that have crossed the fence and whether or not the cross
  /// product of the intersection was positive.
  /// This determines if the fence was crossed one way or another.
  pub crossed: HashMap<Entity, bool>,
}


impl Fence {
  pub fn new(points: Vec<V2>) -> Fence {
    Fence {
      points,
      watching: HashMap::new(),
      crossed: HashMap::new(),
    }
  }

  pub fn segments(&self) -> Vec<(&V2, &V2)> {
    let line1: Vec<&V2> = self.points.iter().collect();
    let line2: Vec<&V2> =
      self.points.iter().collect::<Vec<_>>().drain(1..).collect();
    let segments: Vec<(&V2, &V2)> =
      line1.into_iter().zip(line2.into_iter()).collect();
    segments
  }
}


impl Component for Fence {
  type Storage = HashMapStorage<Self>;
}


/// A special fence that when crossed either increments or decrements an entity's
/// ZLevel. This is a bit of a hack to allow creatures to move up stairs and still
/// render properly.
#[derive(Debug, Clone)]
pub struct StepFence(pub Fence);


impl Component for StepFence {
  type Storage = HashMapStorage<Self>;
}


pub struct FenceSystem;


impl FenceSystem {
  pub fn run_fence(
    &self,
    aabb_tree: &AABBTree,
    entities: &Entities,
    fence_ent: Entity,
    fence: &mut Fence,
    pos: V2,
    velocities: &ReadStorage<Velocity>,
  ) {
    // Clear out our entities this frame
    let last_watching: HashMap<Entity, V2> = fence.watching.drain().collect();
    fence.watching = HashMap::new();
    fence.crossed = HashMap::new();
    let segments: Vec<(V2, V2)> = fence
      .segments()
      .iter()
      .map(|tup| (*tup.0, *tup.1))
      .collect::<Vec<_>>()
      .drain(..)
      .collect();
    // Maintain a list of entities we've already known have crossed
    for (p1, p2) in segments {
      // The fence's points are relative to the fence's position.
      let point1 = p1 + pos;
      let point2 = p2 + pos;
      // Find the radius^2 of our query
      // (length of the segment)^2
      let radius = p1.distance_to(&p2);
      let radius2 = radius * radius;
      // Use the circle that includes the whole segment to query for interesting
      // subjects
      let ebs = aabb_tree.rtree.lookup_in_circle(&point1, &radius2);
      // Insert all the entities we're watching
      for EntityBounds { entity_id, bounds } in ebs {
        let entity = entities.entity(*entity_id);
        if fence_ent == entity {
          continue;
        }
        // Add this thing so we can check it next frame
        let entity_center = AABB::from_mbr(bounds).center();
        fence.watching.insert(entity, entity_center);
        // Continue on to the next entity if we already know this one crossed
        if fence.crossed.contains_key(&entity) {
          continue;
        }
        let entity_velocity = velocities.get(entity);
        // In order to cross a fence a thing must be moving
        if entity_velocity.is_none() {
          continue;
        }
        if let Some(prev_center) = last_watching.get(&entity) {
          // We were watching this entity previously, so check to see if the
          // line made by its previous position and new position intersects with
          // our segment.
          let fence_segment = LineSegment::new(point1, point2);
          let ent_segment = LineSegment::new(*prev_center, entity_center);
          let intersection_point = fence_segment.intersection_with(ent_segment);
          if intersection_point.is_some() {
            // It intersects, so now figure out the cross product
            let vector_moved = ent_segment.vector_difference();
            let vector_fence = fence_segment.vector_difference();
            let cross = vector_fence.cross(vector_moved);
            fence.crossed.insert(entity, cross < 0.0);
          }
        }
      }
    }
  }
}


impl<'a> System<'a> for FenceSystem {
  type SystemData = (
    Read<'a, AABBTree>,
    Entities<'a>,
    WriteStorage<'a, Fence>,
    ReadStorage<'a, Position>,
    WriteStorage<'a, StepFence>,
    ReadStorage<'a, Velocity>,
    WriteStorage<'a, ZLevel>,
  );

  fn run(
    &mut self,
    (
      aabb_tree,
      entities,
      mut fences,
      positions,
      mut step_fences,
      velocities,
      mut zlevels,
    ): Self::SystemData,
  ) {
    // Run regular fences
    for (fence_ent, mut fence, &Position(pos)) in
      (&entities, &mut fences, &positions).join()
    {
      self.run_fence(
        &aabb_tree,
        &entities,
        fence_ent,
        &mut fence,
        pos,
        &velocities,
      );
    }

    // Run step fences
    for (fence_ent, step_fence, &Position(pos)) in
      (&entities, &mut step_fences, &positions).join()
    {
      self.run_fence(
        &aabb_tree,
        &entities,
        fence_ent,
        &mut step_fence.0,
        pos,
        &velocities,
      );

      // run through all crossings and adjust their zlevel
      for (entity, is_positive) in step_fence.0.crossed.iter() {
        zlevels.get_mut(*entity).map(|z| {
          let inc = if *is_positive { 1.0 } else { -1.0 };
          z.0 += inc;
          println!("Stepping z {:?} to {:?}", inc, z.0);
        });
      }
    }
  }
}
