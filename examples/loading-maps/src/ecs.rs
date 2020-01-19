use old_gods::prelude::*;
use web_sys::CanvasRenderingContext2d;

mod render;
use render::HtmlResources;


pub struct ECS<'a, 'b> {
  dispatcher: Dispatcher<'a, 'b>,
  pub base_url: String,
  pub world: World,
  pub rendering_context: Option<CanvasRenderingContext2d>,
  pub resources: HtmlResources,
}


impl<'a, 'b> ECS<'a, 'b> {
  pub fn new_with(base_url: &str, dispatcher_builder: DispatcherBuilder<'a, 'b>) -> Self {
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
      base_url: base_url.into(),
      rendering_context: None,
      resources: HtmlResources::new()
    }
  }

  pub fn new(base_url: &str) -> Self {
    Self::new_with(base_url, DispatcherBuilder::new())
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
    if self.rendering_context.is_some() {
      let mut context =
        self
        .rendering_context
        .take();
      context
        .iter_mut()
        .for_each(|ctx| render::render(&mut self.world, &mut self.resources, ctx));
      self.rendering_context = context;
    } else {
      warn!("no rendering context");
    }
  }
}
