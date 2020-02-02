use old_gods::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};
use wasm_bindgen::{JsCast, UnwrapThrowExt};

mod render;
use render::{
  HtmlResources,
  DebugRenderingData
};

pub use render::RenderingToggles;


pub struct ECS<'a, 'b> {
  dispatcher: Dispatcher<'a, 'b>,
  pub base_url: String,
  debug_mode: bool,
  pre_rendering_context: CanvasRenderingContext2d,
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

    let pre_rendering_context =
      window().unwrap_throw()
      .document().unwrap_throw()
      .create_element("canvas").unwrap_throw()
      .dyn_into::<HtmlCanvasElement>().unwrap_throw()
      .get_context("2d").unwrap_throw().unwrap_throw()
      .dyn_into::<CanvasRenderingContext2d>().unwrap_throw();

    ECS{
      dispatcher,
      world,
      base_url: base_url.into(),
      debug_mode: false,
      rendering_context: None,
      resources: HtmlResources::new(),
      pre_rendering_context
    }
  }

  pub fn set_debug_mode(&mut self, debug: bool) {
    self.debug_mode = debug;
    if debug {
      <DebugRenderingData as SystemData>::setup(&mut self.world);
    }
  }

  /// Set the width and height of the rendering context.
  /// This does not set the width and height of the canvas, instead it sets the
  /// width and height of the inner rendering context. The inner context is the
  /// one that the map is rendered to first. That context is then rendered to
  /// fit inside the outer canvas while maintaining the aspect ratio set by this
  /// function.
  pub fn set_resolution(&self, w: u32, h: u32) {
    let canvas:HtmlCanvasElement =
      self
      .pre_rendering_context
      .canvas().unwrap_throw();
    canvas.set_width(w);
    canvas.set_height(h);
  }

  /// Get the current resolution.
  /// This is the width and height of the inner rendering context.
  pub fn get_resolution(&self) -> (u32, u32) {
    let canvas:HtmlCanvasElement =
      self
      .pre_rendering_context
      .canvas().unwrap_throw();
    (canvas.width(), canvas.height())
  }

  pub fn is_debug(&self) -> bool {
    self.debug_mode
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
        .for_each(|ctx| {
          render::render(&mut self.world, &mut self.resources, ctx);
          if self.debug_mode {
            render::render_debug(&mut self.world, &mut self.resources, ctx);
          }
        });
      self.rendering_context = context;
    } else {
      warn!("no rendering context");
    }
  }
}
