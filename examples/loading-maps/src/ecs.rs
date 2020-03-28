use old_gods::prelude::{
  MapLoadingSystem,
  ScreenSystem,
  ActionSystem,
  ScriptSystem,
  SpriteSystem,
  PlayerSystem,
  Physics,
  AnimationSystem,
  InventorySystem,
  EffectSystem,
  ItemSystem,
  ZoneSystem,
  WarpSystem,
  FenceSystem,
  TweenSystem,

  AABB,
  FPSCounter,
  Screen,
  V2,

  World,
  WorldExt,
  Dispatcher,
  DispatcherBuilder,
  SystemData
}; 
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
  pub world: World,
  pub pre_rendering_context: CanvasRenderingContext2d,
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
      pre_rendering_context,
      resources: HtmlResources::new(),
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
  pub fn set_resolution(&mut self, w: u32, h: u32) {
    let mut screen =
      self
      .world
      .write_resource::<Screen>();
    if let Some(canvas) = &mut self.pre_rendering_context.canvas() {
      canvas.set_width(w);
      canvas.set_height(h);
    }
  }

  /// Get the current resolution.
  /// This is the width and height of the inner rendering context.
  pub fn get_resolution(&self) -> (u32, u32) {
    self
      .world
      .read_resource::<Screen>()
      .window_size
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

  // TODO: Separate #rendering into three steps: 
  // * Render the map into the pre-context, then aspect fit render to main
  // * Render debug stuff onto main
  // * Render UI onto main
  pub fn render(&mut self) {
    let mut may_ctx = self.rendering_context.take();
    if let Some(ctx) = &mut may_ctx {
      render::render(
        &mut self.world,
        &mut self.resources,
        &mut self.pre_rendering_context
      );

      if self.debug_mode {
        render::render_debug(
          &mut self.world,
          &mut self.resources,
          &mut self.pre_rendering_context
        );
      }

      let canvas =
        self
        .pre_rendering_context
        .canvas()
        .unwrap_throw();

      let window =
        ctx
        .canvas()
        .unwrap_throw();

      let map_size = V2::new(canvas.width() as f32, canvas.height() as f32);
      let win_size = V2::new(window.width() as f32, window.height() as f32); 

      // Aspect fit our pre_rendering_context inside the final rendering_context 
      let src = AABB::new(
        0.0, 0.0,
        map_size.x, map_size.y 
      );

      let dest = AABB::aabb_to_aspect_fit_inside(map_size, win_size).round(); 
      trace!("drawing {:#?} to {:#?}", src, dest);

      ctx
        .draw_image_with_html_canvas_element_and_dw_and_dh(
          &canvas,
          dest.top_left.x as f64,
          dest.top_left.y as f64,
          dest.width() as f64,
          dest.height() as f64
        )
        .unwrap_throw();
    } else {
      warn!("no rendering context");
    }
    self.rendering_context = may_ctx;
  }
}
