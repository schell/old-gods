//! The WebEngine definition.
//!
//! TODO: Abstract engine details into a trait
//! TODO: Rename this module WebEngine that implements Engine
use old_gods::prelude::{
  ActionSystem, AnimationSystem, Dispatcher, DispatcherBuilder, EffectSystem,
  FPSCounter, FenceSystem, GamepadSystem, InventorySystem, ItemSystem,
  MapLoadingSystem, Physics, PlayerSystem, Screen, ScreenSystem, ScriptSystem,
  SpriteSystem, SystemData, TweenSystem, WarpSystem, World, WorldExt,
  ZoneSystem, AABB, V2,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

mod render;
use render::{DebugRenderingData, HtmlResources};

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
  pub fn new_with(
    base_url: &str,
    dispatcher_builder: DispatcherBuilder<'a, 'b>,
  ) -> Self {
    let mut world = World::new();
    let mut dispatcher = dispatcher_builder
      //.with_thread_local(SoundSystem::new())
      .with_thread_local(MapLoadingSystem { opt_reader: None })
      .with_thread_local(ScreenSystem)
      .with_thread_local(ActionSystem)
      .with_thread_local(ScriptSystem)
      .with_thread_local(SpriteSystem)
      .with_thread_local(PlayerSystem)
      .with_thread_local(Physics::new())
      .with_thread_local(AnimationSystem)
      .with_thread_local(InventorySystem)
      .with_thread_local(EffectSystem)
      .with_thread_local(ItemSystem)
      .with_thread_local(ZoneSystem)
      .with_thread_local(WarpSystem)
      .with_thread_local(FenceSystem)
      .with_thread_local(TweenSystem)
      .with_thread_local(GamepadSystem::new())
      .build();

    dispatcher.setup(&mut world);

    // Maintain once so all our resources are created.
    world.maintain();

    let pre_rendering_context = window()
      .unwrap_throw()
      .document()
      .unwrap_throw()
      .create_element("canvas")
      .unwrap_throw()
      .dyn_into::<HtmlCanvasElement>()
      .unwrap_throw()
      .get_context("2d")
      .unwrap_throw()
      .unwrap_throw()
      .dyn_into::<CanvasRenderingContext2d>()
      .unwrap_throw();

    ECS {
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
    if let Some(canvas) = &mut self.pre_rendering_context.canvas() {
      canvas.set_width(w);
      canvas.set_height(h);
    }
  }

  /// Get the current resolution.
  /// This is the width and height of the inner rendering context.
  pub fn get_resolution(&self) -> (u32, u32) {
    self.world.read_resource::<Screen>().window_size
  }

  pub fn is_debug(&self) -> bool {
    self.debug_mode
  }

  pub fn new(base_url: &str) -> Self {
    Self::new_with(base_url, DispatcherBuilder::new())
  }

  pub fn maintain(&mut self) {
    self.world.write_resource::<FPSCounter>().next_frame();

    self.dispatcher.dispatch(&mut self.world);

    self.world.maintain();
  }

  // TODO: Separate #rendering into three steps:
  // * Render the map into the pre-context, then aspect fit render to main
  // * Render debug stuff onto main
  // * Render UI onto main
  pub fn render(&mut self) {
    let mut may_ctx = self.rendering_context.take();
    if let Some(mut ctx) = may_ctx.as_mut() {
      render::render_map(
        &mut self.world,
        &mut self.resources,
        &mut self.pre_rendering_context,
      );

      if self.debug_mode {
        render::render_map_debug(
          &mut self.world,
          &mut self.resources,
          &mut self.pre_rendering_context,
        );
      }

      let canvas = self.pre_rendering_context.canvas().unwrap_throw();

      let window = ctx.canvas().unwrap_throw();

      // Aspect fit our pre_rendering_context inside the final rendering_context
      let map_size = V2::new(canvas.width() as f32, canvas.height() as f32);
      let win_size = V2::new(window.width() as f32, window.height() as f32);
      let dest = AABB::aabb_to_aspect_fit_inside(map_size, win_size).round();
      ctx
        .draw_image_with_html_canvas_element_and_dw_and_dh(
          &canvas,
          dest.top_left.x as f64,
          dest.top_left.y as f64,
          dest.width() as f64,
          dest.height() as f64,
        )
        .unwrap_throw();

      // Draw the UI
      render::render_ui(&mut self.world, &mut self.resources, &mut ctx);
      if self.debug_mode {
        render::render_ui_debug(&mut self.world, &mut self.resources, &mut ctx);
      }
    } else {
      warn!("no rendering context");
    }
    self.rendering_context = may_ctx;
  }
}
