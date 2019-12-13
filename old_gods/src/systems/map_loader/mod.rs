use shrev::EventChannel;
use specs::prelude::*;
use std::collections::HashSet;

use super::super::{
  geom::V2,
  color::BackgroundColor
};

pub mod load;
pub use load::*;

mod map_loader;
pub use self::map_loader::{MapLoader, LoadedLayers};


////////////////////////////////////////////////////////////////////////////////
/// MapLoadingEvent
////////////////////////////////////////////////////////////////////////////////
pub struct Tags(pub HashSet<String>);


impl Component for Tags {
  type Storage = HashMapStorage<Self>;
}
////////////////////////////////////////////////////////////////////////////////
/// MapLoadingEvent
////////////////////////////////////////////////////////////////////////////////
pub enum MapLoadingEvent {
  /// Load a new map into the ECS
  LoadMap(String, V2),
  LoadSprite{
    file: String,
    variant: String,
    keyframe: Option<String>,
    origin: V2
  },
  UnloadEverything
}


////////////////////////////////////////////////////////////////////////////////
/// MapLoadingSystem
////////////////////////////////////////////////////////////////////////////////
pub struct MapLoadingSystem {
  pub opt_reader: Option<ReaderId<MapLoadingEvent>>,
}


impl<'a> System<'a> for MapLoadingSystem {
  type SystemData = (
    Read<'a, EventChannel<MapLoadingEvent>>,
    Write<'a, BackgroundColor>,
    Read<'a, LazyUpdate>
  );

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);

    self.opt_reader = Some(
      world
        .fetch_mut::<EventChannel<MapLoadingEvent>>()
        .register_reader()
    );
  }

  fn run(
    &mut self,
    (
      chan,
      mut background_color,
      lazy
    ): Self::SystemData
  ) {
    if let Some(reader) = self.opt_reader.as_mut() {
      for event in chan.read(reader) {
        match event {
          MapLoadingEvent::LoadMap(file, _global_pos) => {
            let file = file.clone();
            MapLoader::load_it(file, &lazy);
          }
          MapLoadingEvent::LoadSprite{.. /*file, variant, keyframe, origin*/} => {
            //let mut loader = MapLoader::new(&entities, &update);
            //loader.load_sprite(&file, &variant, keyframe.as_ref(), None, &origin);
          }
          MapLoadingEvent::UnloadEverything => {
            lazy
              .exec_mut(|world| world.delete_all());
            background_color.0 = BackgroundColor::default().0;
          }
        }
      }
    }
  }
}
