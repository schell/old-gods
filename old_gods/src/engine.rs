//! The engine context.
use shrev::EventChannel;
use specs::{DispatcherBuilder, World};
use std::any::Any;

use super::color::Color;
use super::geom::V2;
use super::systems::{
  //sound::SoundSystem,
  map_loader::{
    //MapLoadingSystem,
    MapLoadingEvent,
  },
  /*screen::ScreenSystem,
   *action::ActionSystem,
   *script::ScriptSystem,
   *sprite::SpriteSystem,
   *player::PlayerSystem,
   *physics::Physics,
   *animation::AnimationSystem,
   *inventory::InventorySystem,
   *effect::EffectSystem,
   *item::ItemSystem,
   *zone::ZoneSystem,
   *sprite::WarpSystem,
   *fence::FenceSystem,
   *tween::TweenSystem */
};

pub trait Canvas {
  fn set_draw_color(c: Color);
  fn clear();
  fn present();
}


pub trait ResourceLoader {}


pub struct EngineLoopBuilder<'a, 'b, X: Any> {
  file: Option<String>,
  dispatcher_builder: Option<DispatcherBuilder<'a, 'b>>,
  setup: Option<Box<dyn Fn(&mut World)>>,
  exit: Option<Box<dyn Fn(&mut World) -> Option<X>>>,
}


impl<'a, 'b, X: Any> EngineLoopBuilder<'a, 'b, X> {
  pub fn new() -> Self {
    EngineLoopBuilder {
      file: None,
      dispatcher_builder: None,
      setup: None,
      exit: None,
    }
  }

  pub fn with_file<I: Into<String>>(self, file: I) -> Self {
    let mut b = self;
    b.file = Some(file.into());
    b
  }

  pub fn with_dispatcher_builder(
    self,
    builder: DispatcherBuilder<'a, 'b>,
  ) -> Self {
    let mut b = self;
    b.dispatcher_builder = Some(builder);
    b
  }

  pub fn with_setup<F: Fn(&mut World) + 'static>(self, f: F) -> Self {
    let mut b = self;
    b.setup = Some(Box::new(f));
    b
  }

  pub fn with_exit<F>(self, f: F) -> Self
  where
    F: Fn(&mut World) -> Option<X> + 'static,
  {
    let mut b = self;
    b.exit = Some(Box::new(f));
    b
  }

  //pub fn run<E:Engine>(self, engine: &E) -> X {
  //  let mut builder = self;
  //  let file = builder.file.take();
  //  let setup = builder.setup.take();
  //  let exit = builder.exit.take();
  //  let dispatcher_builder = builder.dispatcher_builder.take();
  //  // Create the world
  //  let mut world = World::new();

  //  let mut dispatcher =
  //    dispatcher_builder
  //      .unwrap_or(DispatcherBuilder::new())
  //    //.with_thread_local(SoundSystem::new())
  //      .with(MapLoadingSystem{ opt_reader: None }, "map", &[])
  //      .with(ScreenSystem, "screen", &[])
  //      .with(ActionSystem, "action", &[])
  //      .with(ScriptSystem, "script", &["action"])
  //      .with(SpriteSystem, "sprite", &["script"])
  //      .with(PlayerSystem, "control", &[])
  //      .with(Physics::new(), "physics", &[])
  //      .with(AnimationSystem, "animation", &[])
  //      .with(InventorySystem, "inventory", &[])
  //      .with(EffectSystem, "effect", &[])
  //      .with(ItemSystem, "item", &["action", "effect"])
  //      .with(ZoneSystem, "zone", &[])
  //      .with(WarpSystem, "warp", &["physics"])
  //      .with(FenceSystem, "fence", &["physics"])
  //      .with(TweenSystem, "tween", &[])
  //      .build();

  //  dispatcher
  //    .setup(&mut world.res);

  //  // Maintain once so all our resources are created.
  //  world
  //    .maintain();

  //  // Give the library user a chance to set up their world
  //  setup
  //    .into_iter()
  //    .for_each(|bx:Box<dyn Fn(&mut World)>| {
  //      bx(&mut world);
  //    });

  //  // Load our map file
  //  file
  //    .into_iter()
  //    .for_each(|file| {
  //      world
  //        .write_resource::<EventChannel<MapLoadingEvent>>()
  //        .single_write(MapLoadingEvent::LoadMap(file.to_string(), V2::new(0.0, 0.0)));
  //    });

  //  let mut exit_value = None;

  //  'running: loop {
  //    {
  //      // Update the delta (and FPS)
  //      //world
  //      //  .write_resource::<FPSCounter>()
  //      //  .next_frame();
  //    }

  //    dispatcher
  //      .dispatch(&mut world.res);
  //    world
  //      .maintain();

  //    //if world.read_resource::<UI>().should_reload() {
  //    //  let mut map_chan =
  //    //    world
  //    //    .write_resource::<EventChannel<MapLoadingEvent>>();
  //    //  map_chan
  //    //    .single_write(MapLoadingEvent::UnloadEverything);
  //    //  file
  //    //    .map(|file| {
  //    //      map_chan
  //    //        .single_write(MapLoadingEvent::LoadMap(
  //    //          file.to_string(),
  //    //          V2::new(0.0, 0.0)
  //    //        ));
  //    //    });
  //    //}

  //    // Check if the loop should end
  //    exit
  //      .iter()
  //      .for_each(|bx| {
  //        exit_value = bx(&mut world);
  //      });

  //    if exit_value.is_some() {
  //      return exit_value.unwrap();
  //    } else {
  //      engine.render(&mut world);
  //    }
  //    // Render everything
  //    //let data:<RenderingSystem as System>::SystemData =
  //    //  SystemData::fetch(&world.res);
  //    //rendering_system
  //    //  .run(data);
  //  }
  //}
}

pub trait Engine<'a, 'b>
where
  Self: Sized,
{
  type Canvas;
  type ResourceLoader;

  fn new_with(dispatcher_builder: DispatcherBuilder<'a, 'b>) -> Self;

  fn new() -> Self {
    Self::new_with(DispatcherBuilder::new())
  }

  fn world(&self) -> &World;
  fn world_mut(&mut self) -> &mut World;

  fn unload_map(&mut self) {
    let world = self.world();
    let mut map_chan = world.fetch_mut::<EventChannel<MapLoadingEvent>>();
    map_chan.single_write(MapLoadingEvent::UnloadEverything);
  }

  fn load_map<X: Into<String>>(&mut self, file: X) {
    let file = file.into();
    let world = self.world();
    let mut map_chan = world.fetch_mut::<EventChannel<MapLoadingEvent>>();
    map_chan.single_write(MapLoadingEvent::UnloadEverything);
    map_chan.single_write(MapLoadingEvent::LoadMap(
      file.to_string(),
      V2::new(0.0, 0.0),
    ));
  }

  //fn put_canvas(&mut self, c: Self::Canvas);
  //fn take_canvas(&mut self) -> Self::Canvas;

  //fn put_loader(&mut self, r: Self::ResourceLoader);
  //fn take_loader(&mut self) ->  Self::ResourceLoader;

  fn render(&self, world: &mut World);
}
