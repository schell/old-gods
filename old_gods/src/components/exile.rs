use specs::prelude::{Component, Entity, WriteStorage, HashMapStorage};
use std::collections::HashSet;


/// ## Exiled entities

/// Since multiple systems may want to exile or domesticate (un-exile) an entity
/// we use a string to associate an exile with what has exiled it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExiledBy(pub String);


/// An exiled entity is effectively removed from the game, but still exists in
/// the ECS. This maintains the exiled entity's comonents. It's the various other
/// systems' responsibility to check entities for their Exiled compnonents, or
/// lack thereof.
#[derive(Debug, Clone)]
pub struct Exile(pub HashSet<ExiledBy>);


impl Component for Exile {
  type Storage = HashMapStorage<Exile>;
}


impl Exile {
  pub fn exile(entity: Entity, by: &str, exiles: &mut WriteStorage<Exile>) {
    let by = ExiledBy(by.to_owned());
    if exiles.contains(entity) {
      let set = exiles.get_mut(entity).expect("This should never happen.");
      set.0.insert(by);
    } else {
      let mut set = HashSet::new();
      set.insert(by);
      exiles
        .insert(entity, Exile(set))
        .expect("Could not insert an Exile set.");
    }
  }

  pub fn domesticate(
    entity: Entity,
    by: &str,
    exiles: &mut WriteStorage<Exile>,
  ) {
    let by = ExiledBy(by.to_owned());
    if exiles.contains(entity) {
      let set = {
        let set = exiles.get_mut(entity).expect("This should never happen.");
        set.0.remove(&by);
        set.clone()
      };
      if set.0.is_empty() {
        exiles.remove(entity);
      }
    }
  }
}
