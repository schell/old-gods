use specs::prelude::{
  Component, Entities, Entity, Join, ReadStorage, World, WorldExt, WriteStorage,
};

pub use super::geom::*;
pub use super::systems::action::{
  Action, FitnessStrategy, Lifespan, TakeAction,
};
pub use super::systems::animation::Animation;
pub use super::systems::effect::Effect;
pub use super::systems::fence::{Fence, StepFence};
pub use super::systems::inventory::{Inventory, Looting};
pub use super::systems::item::Item;
pub use super::systems::physics::{Barrier, Position, Velocity};
pub use super::systems::player::{MaxSpeed, Player, SuspendPlayer, AI};
//pub use super::systems::rendering::{ZLevel, Rendering, OriginOffset};
pub use super::systems::script::Script;
//pub use super::systems::sound::{Music, Sound};
pub use super::systems::sprite::{Exile, ExiledBy, Sprite};
pub use super::systems::tween::{Easing, Tween, TweenParam};
//pub use super::systems::ui::Cardinal;
pub use super::systems::zone::Zone;

mod attributes;
pub use self::attributes::*;

mod rendering;
pub use self::rendering::*;

mod cardinal;
pub use self::cardinal::*;

mod font_details;
pub use self::font_details::*;


/// Find a component and entity by another component
pub fn find_by<A, B>(world: &World, a: &A) -> Option<(Entity, B)>
where
  A: Component + PartialEq,
  B: Component + Clone,
{
  let a_store = world.read_storage::<A>();
  let b_store = world.read_storage::<B>();
  let ents = world.entities();
  for (e, a_ref, b_ref) in (&ents, &a_store, &b_store).join() {
    if *a_ref == *a {
      let b = (*b_ref).clone();
      return Some((e, b));
    }
  }
  None
}

/// Allows `get` and `contains` on read or write storages.
pub trait GetStorage<T> {
  fn get(&self, e: Entity) -> Option<&T>;

  fn contains(&self, e: Entity) -> bool;
}


impl<'a, T: Component> GetStorage<T> for WriteStorage<'a, T> {
  fn get(&self, e: Entity) -> Option<&T> {
    self.get(e)
  }

  fn contains(&self, e: Entity) -> bool {
    self.contains(e)
  }
}


impl<'a, T: Component> GetStorage<T> for ReadStorage<'a, T> {
  fn get(&self, e: Entity) -> Option<&T> {
    self.get(e)
  }

  fn contains(&self, e: Entity) -> bool {
    self.contains(e)
  }
}


/// Returns the position/current location of the entity, offset by any
/// OriginOffset or barrier center it may also have.
pub fn entity_location<P, O>(
  ent: Entity,
  positions: &P,
  origins: &O,
) -> Option<V2>
where
  P: GetStorage<Position>,
  O: GetStorage<OriginOffset>,
{
  let pos = positions.get(ent).map(|p| p.0.clone());
  let origin = origins
    .get(ent)
    .map(|o| o.0.clone())
    .unwrap_or(V2::origin());
  pos.map(|p| p + origin)
}


/// Returns the entities origin offset or barrier center.
pub fn entity_local_origin<O, S>(ent: Entity, shapes: &S, origins: &O) -> V2
where
  S: GetStorage<Shape>,
  O: GetStorage<OriginOffset>,
{
  origins
    .get(ent)
    .map(|o| o.0)
    .or(
      // try to locate a shape - if it has a shape we will consider
      // the center of its aabb as the origin offset.
      shapes.get(ent).map(|s| s.aabb().center()),
    )
    .unwrap_or(V2::origin())
}


/// Used for testing the number of entities before and after a function is run.
pub fn with_ent_counts<F: FnMut(), G: Fn(u32, u32)>(
  entities: &Entities,
  mut f: F,
  g: G,
) {
  let before_entity_count = &entities.join().fold(0, |n, _| n + 1);
  f();
  let after_entity_count = &entities.join().fold(0, |n, _| n + 1);
  g(*before_entity_count, *after_entity_count);
}
