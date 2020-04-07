use serde_json::Value;
use specs::prelude::*;
use std::collections::HashMap;
use std::marker::PhantomData;

use super::super::components::{Action, Exile, Inventory, Name, Sprite};
//mod container;
mod door;

use container::Container;
use door::Door;


/// A script component gives the entity special behavior without adding any
/// new data.
#[derive(Debug, Clone)]
pub enum Script {
  /// The entity is a sprite that acts like a container, being able to be opened,
  /// closed and looted.
  Container,

  /// The entity is a sprite that acts like a door, being able to be opened and
  /// closed - changing the barriers within it.
  Door,

  /// Some other script that will be taken care of by another system
  Other {
    /// The name of this script
    name: String,

    /// Any special properties this script may have
    properties: HashMap<String, Value>,
  },
}


impl Script {
  pub fn tiled_key() -> String {
    "script".to_string()
  }

  pub fn from_str(
    s: &str,
    props: Option<HashMap<String, Value>>,
  ) -> Result<Script, String> {
    match s {
      "container" => Ok(Script::Container),
      "door" => Ok(Script::Door),
      "" => Err("Object script may not be empty".to_string()),
      s => Ok(Script::Other {
        name: s.to_string(),
        properties: props.unwrap_or(HashMap::new()),
      }),
    }
  }

  /// Return the contained string in the "Other" case, if possible.
  pub fn other_string(&self) -> Option<&String> {
    self.other().map(|(n, _)| n)
  }

  /// Return the other script if possible
  pub fn other(&self) -> Option<(&String, &HashMap<String, Value>)> {
    match self {
      Script::Other { name, properties } => Some((name, properties)),
      _ => None,
    }
  }
}


impl Component for Script {
  type Storage = HashMapStorage<Self>;
}


trait ScriptFromKey
where
  Self: std::any::Any + Sized
{
  type Storage;

  fn insert(key: &str) -> Option<Self>;
}


pub struct ScriptSystem<S> {
  phantom: PhantomData<S>
}


impl<S:ScriptFromKey> ScriptSystem<S> {
  pub fn new() -> Self {
    ScriptSystem {
      phantom: PhantomData
    }
  }
}


impl<'a, S:ScriptFromKey> System<'a> for ScriptSystem<S> {
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    WriteStorage<'a, Script>,
  );

  fn run(
    &mut self,
    (
      entities,
      lazy,
      mut scripts,
    ): Self::SystemData,
  ) {
    for (ent, script, sprite, ()) in
      (&entities, &scripts, &sprites, !&exiles).join()
    {
      match script {
        Script::Container => {
          Container::run(
            &actions,
            &entities,
            ent,
            &inventories,
            &lazy,
            &names,
            sprite,
          );
        }

        Script::Door => {
          Door::run(&actions, &entities, ent, &lazy, sprite);
        }

        Script::Other { name, .. } => {
          println!("Seeing sprite with script {:?}", name);
        }
      }
    }

    for (_ent, script, ()) in (&entities, &scripts, !&exiles).join() {
      match script {
        Script::Other { name, .. } => {
          println!("Seeing object with script {:?}", name);
        }
        _ => {}
      }
    }
  }
}
