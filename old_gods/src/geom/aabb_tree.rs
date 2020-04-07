pub use spade::rtree::NearestNeighborIterator;
use spade::rtree::RTree;
/// # Things that live in an RTree.
/// The component itself is an AABB in global 2d space.
use spade::{BoundingRect, SpatialObject};
use specs::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;

use super::super::prelude::{Barrier, GetStorage, Position, Shape, AABB, V2};


////////////////////////////////////////////////////////////////////////////////
/// ## EntityBounds
/// Entities with AABB boundaries.
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
pub struct EntityBounds {
  pub entity_id: u32,
  pub bounds: BoundingRect<V2>,
}

impl SpatialObject for EntityBounds {
  type Point = V2;

  fn mbr(&self) -> BoundingRect<V2> {
    self.bounds.mbr()
  }

  fn distance2(&self, point: &V2) -> f32 {
    self.bounds.distance2(point)
  }
}


////////////////////////////////////////////////////////////////////////////////
/// ## The AABBTree structure
////////////////////////////////////////////////////////////////////////////////
pub struct AABBTree {
  pub index: HashMap<u32, AABB>,
  pub rtree: RTree<EntityBounds>,
}


impl Default for AABBTree {
  fn default() -> AABBTree {
    AABBTree::new()
  }
}


impl AABBTree {
  pub fn new() -> AABBTree {
    let rtree = RTree::new();
    let index = HashMap::new();
    AABBTree { rtree, index }
  }


  pub fn insert(&mut self, entity: Entity, aabb: AABB) {
    let id = entity.id();
    if self.index.contains_key(&id) {
      // We have to delete first
      self.remove(entity);
    }
    self.index.insert(id, aabb);
    self.rtree.insert(EntityBounds {
      entity_id: id,
      bounds: aabb.to_mbr(),
    });
  }


  pub fn remove(&mut self, entity: Entity) {
    let id = entity.id();
    if let Some(aabb) = self.index.remove(&id) {
      let mut removals = 0;
      let eb = EntityBounds {
        entity_id: id,
        bounds: aabb.to_mbr(),
      };
      while self.rtree.remove(&eb) {
        removals += 1;
      }
      if removals == 0 {
        panic!("Could not remove any AABBs from the rtree.");
      }
    }
  }


  /// Query for any aabbs intersecting the given aabb. Filter the results
  /// to *not* include aabbs of the given entity.
  ///
  /// Note: AABBs that are colinear along some axis with the query AABB will be
  /// returned.
  ///
  /// Note: Point AABBs (i.e. with zero value extents, zero width and zero height)
  /// will be returned.
  ///
  /// Note: The results are sorted in ascending order of their distance from the center
  /// of the query aabb.
  ///
  /// ```
  /// extern crate specs;
  /// extern crate engine;
  /// use engine::geom::AABB;
  /// use engine::geom::V2;
  /// use engine::systems::rtree::AABBTree;
  /// use specs::prelude::*;
  ///
  /// let mut world = World::new();
  /// let mut tree = AABBTree::new();
  /// let ent = world.create_entity().build();
  /// let aabb = AABB {
  ///   top_left: V2::new(0.0, 0.0),
  ///   extents: V2::new(10.0, 10.0),
  /// };
  /// tree.insert(ent, aabb);
  ///
  /// let colinear_ent = world.create_entity().build();
  /// let colinear_aabb = AABB {
  ///   top_left: V2::new(10.0, 0.0),
  ///   extents: V2::new(10.0, 10.0),
  /// };
  /// tree.insert(colinear_ent, colinear_aabb);
  ///
  /// let point_ent = world.create_entity().build();
  /// let point = V2::new(5.0, 5.0);
  /// let point_aabb = AABB {
  ///   top_left: point,
  ///   extents: V2::origin(),
  /// };
  /// tree.insert(point_ent, point_aabb);
  ///
  /// let intersections = tree.query(&world.entities(), &aabb, &ent);
  /// let expected_result =
  ///   vec![(point_ent, point_aabb), (colinear_ent, colinear_aabb)];
  /// assert_eq!(expected_result, intersections);
  /// ```
  pub fn query(
    &self,
    entities: &Entities,
    aabb: &AABB,
    filter_entity: &Entity,
  ) -> Vec<(Entity, AABB)> {
    let aabb_center = aabb.center();
    let mut collisions: Vec<&EntityBounds> = self
      .rtree
      .lookup_in_rectangle(&aabb.to_mbr())
      .into_iter()
      .filter(|eb| eb.entity_id != filter_entity.id())
      .collect();

    collisions.sort_by(|a, b| {
      let ad = a.distance2(&aabb_center);
      let bd = b.distance2(&aabb_center);
      if ad < bd {
        Ordering::Less
      } else if ad > bd {
        Ordering::Greater
      } else {
        Ordering::Equal
      }
    });

    collisions
      .into_iter()
      .map(|eb| (entities.entity(eb.entity_id), AABB::from_mbr(&eb.bounds)))
      .collect()
  }

  /// Returns a vector of entity, shape and the minimum translation vector
  /// needed to push the shape out of intersection.
  pub fn query_intersecting_shapes<S, P>(
    &self,
    entities: &Entities,
    entity: &Entity,
    shapes: &S,
    positions: &P,
  ) -> Vec<(Entity, Shape, V2)>
  where
    S: GetStorage<Shape>,
    P: GetStorage<Position>,
  {
    let no_barriers: Option<&ReadStorage<Barrier>> = None;
    self.query_intersecting(entities, entity, shapes, positions, no_barriers)
  }

  /// Like `query_intersecting_shapes` but the results only include entities with barriers.
  /// Returns a vector of entity, shape and the minimum translation vector
  /// needed to push the shape out of intersection.
  pub fn query_intersecting_barriers<S, P, B>(
    &self,
    entities: &Entities,
    entity: &Entity,
    shapes: &S,
    positions: &P,
    barriers: &B,
  ) -> Vec<(Entity, Shape, V2)>
  where
    S: GetStorage<Shape>,
    P: GetStorage<Position>,
    B: GetStorage<Barrier>,
  {
    self.query_intersecting(entities, entity, shapes, positions, Some(barriers))
  }

  fn query_intersecting<S, P, B>(
    &self,
    entities: &Entities,
    entity: &Entity,
    shapes: &S,
    positions: &P,
    may_barriers: Option<&B>,
  ) -> Vec<(Entity, Shape, V2)>
  where
    S: GetStorage<Shape>,
    P: GetStorage<Position>,
    B: GetStorage<Barrier>,
  {
    let shape = shapes.get(*entity);
    let pos = positions.get(*entity);
    let include_by_barrier =
      |ent| may_barriers.is_none() || may_barriers.unwrap().contains(ent);
    let should_include =
      shape.is_some() && pos.is_some() && include_by_barrier(*entity);
    if !should_include {
      return vec![];
    }

    let shape = shape.unwrap();
    let pos = pos.unwrap().0;
    self
      .query(&entities, &shape.aabb().translate(&pos), entity)
      .into_iter()
      .filter_map(|(other_ent, _)| {
        let other_shape = shapes.get(other_ent);

        let other_position = positions.get(other_ent);

        let should_include = other_shape.is_some()
          && other_position.is_some()
          && include_by_barrier(other_ent);
        if !should_include {
          return None;
        }

        let other_shape = other_shape.unwrap();
        shape
          .mtv_apart(pos, &other_shape, other_position.unwrap().0)
          .map(|mtv| (other_ent, other_shape.clone(), mtv))
      })
      .collect()
  }

  /// Query for the n closest things to the point. Filter the results to *not*
  /// include the given entity.
  ///
  /// ```
  /// extern crate specs;
  /// extern crate engine;
  /// use engine::geom::AABB;
  /// use engine::geom::V2;
  /// use engine::systems::rtree::AABBTree;
  /// use specs::prelude::*;
  ///
  /// let mut world = World::new();
  /// let mut tree = AABBTree::new();
  /// let ent = world.create_entity().build();
  /// let point_ent = world.create_entity().build();
  /// let point_aabb = AABB::identity().translate(V2::new(5.0, 5.0));
  /// tree.insert(point_ent, point_aabb);
  /// let nearest = tree.query_nearest_n(&entitities, &V2::origin(), 2, &ent);
  /// assert_eq!(nearest, vec![(point_ent, point_aabb)]);
  /// ```
  pub fn query_nearest_n(
    &self,
    entities: &Entities,
    point: &V2,
    n: usize,
    filter_entity: &Entity,
  ) -> Vec<(Entity, AABB)> {
    self
      .rtree
      .nearest_n_neighbors(point, n + 1)
      .into_iter()
      .filter(|eb| eb.entity_id != filter_entity.id())
      .map(|eb| (entities.entity(eb.entity_id), AABB::from_mbr(&eb.bounds)))
      .collect()
  }

  pub fn update_tree(
    &mut self,
    entities: &Entities,
    events: Vec<&ComponentEvent>,
    get_aabb: impl Fn(Entity) -> Option<(Entity, AABB)>,
  ) {
    for event in events {
      match event {
        ComponentEvent::Inserted(id) => {
          let entity = entities.entity(*id);
          get_aabb(entity).map(|(entity, aabb)| {
            self.insert(entity, aabb);
          });
        }
        ComponentEvent::Modified(id) => {
          let entity = entities.entity(*id);
          get_aabb(entity).map(|(entity, aabb)| {
            self.insert(entity, aabb);
          });
        }
        ComponentEvent::Removed(id) => {
          let entity = entities.entity(*id);
          self.remove(entity);
        }
      }
    }
  }
}
