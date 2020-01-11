use old_gods::prelude::*;
use web_sys::CanvasRenderingContext2d;

mod render;


pub struct ECS<'a, 'b> {
  dispatcher: Dispatcher<'a, 'b>,
  pub world: World,
  pub rendering_context: Option<CanvasRenderingContext2d>
}


impl<'a, 'b> ECS<'a, 'b> {
  pub fn new_with(dispatcher_builder: DispatcherBuilder<'a, 'b>) -> Self {
    let mut world = World::new();
    let mut dispatcher =
      dispatcher_builder
      //.with_thread_local(SoundSystem::new())
      .with(MapLoadingSystem{ opt_reader: None }, "map", &[])
      .with(ScreenSystem, "screen", &[])
      .with(ActionSystem, "action", &[])
      .with(ScriptSystem, "script", &["action"])
      .with(SpriteSystem, "sprite", &["script"])
      .with(PlayerSystem, "control", &[])
      .with(Physics::new(), "physics", &[])
      .with(AnimationSystem, "animation", &[])
      .with(InventorySystem, "inventory", &[])
      .with(EffectSystem, "effect", &[])
      .with(ItemSystem, "item", &["action", "effect"])
      .with(ZoneSystem, "zone", &[])
      .with(WarpSystem, "warp", &["physics"])
      .with(FenceSystem, "fence", &["physics"])
      .with(TweenSystem, "tween", &[])
      .build();

    dispatcher
      .setup(&mut world);

    // Maintain once so all our resources are created.
    world
      .maintain();

    ECS{
      dispatcher,
      world,
      rendering_context: None
    }
  }

  pub fn new() -> Self {
    Self::new_with(DispatcherBuilder::new())
  }

  pub fn maintain(&mut self) {
    self
      .world
      .write_resource::<FPSCounter>()
      .next_frame();

    self
      .dispatcher
      .dispatch(&mut self.world);

    self
      .world
      .maintain();
  }

  pub fn render(&mut self) {
    let mut context =
      self
      .rendering_context
      .take();
    context
      .iter_mut()
      .for_each(|ctx| render::render(&mut self.world, ctx));
    self.rendering_context = context;
  }
}
