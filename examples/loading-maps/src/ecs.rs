//! The WebEngine definition.
//!
//! TODO: Abstract engine details into a trait
//! TODO: Rename this module WebEngine that implements Engine
use old_gods::prelude::{
  AnimationSystem, BackgroundColor, Color, Dispatcher, DispatcherBuilder,
  FPSCounter, GamepadSystem, Physics, PlayerSystem, Screen,
  ScreenSystem, SystemData, World, WorldExt, AABB, V2, TweenSystem
};
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

pub mod render;
pub use render::RenderingToggles;
use render::{DebugRenderingData, HtmlResources};

pub mod resources;
pub mod systems;


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
    world.insert(BackgroundColor(Color::rgb(0, 0, 0)));

    let mut dispatcher = dispatcher_builder
      .with_thread_local(systems::tiled::TiledmapSystem::new(base_url))
      .with_thread_local(Physics::new())
      .with_thread_local(ScreenSystem)
      .with_thread_local(AnimationSystem)
      .with_thread_local(GamepadSystem::new())
      .with_thread_local(PlayerSystem)
      .with_thread_local(systems::inventory::InventorySystem)
      .with_thread_local(TweenSystem)
      //.with_thread_local(SoundSystem::new())
      //.with_thread_local(MapLoadingSystem { opt_reader: None })
      //.with_thread_local(ActionSystem)
      //.with_thread_local(SpriteSystem)
      //.with_thread_local(EffectSystem)
      //.with_thread_local(ItemSystem)
      //.with_thread_local(ZoneSystem)
      //.with_thread_local(FenceSystem)
      .build();

    dispatcher.setup(&mut world);

    // Maintain once so all our resources are created.
    world.maintain();

    let pre_rendering_context = window()
      .expect("no window")
      .document()
      .expect("no document")
      .create_element("canvas")
      .expect("can't create canvas")
      .dyn_into::<HtmlCanvasElement>()
      .expect("can't coerce canvas")
      .get_context("2d")
      .expect("can't call get_context('2d')")
      .expect("can't get canvas rendering context")
      .dyn_into::<CanvasRenderingContext2d>()
      .expect("can't coerce canvas rendering context");


    pre_rendering_context.set_image_smoothing_enabled(false);

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
      let mut screen = self.world.write_resource::<Screen>();
      screen.set_size((w, h));
      canvas.set_width(w);
      canvas.set_height(h);
    }
  }

  /// Get the current resolution.
  /// This is the width and height of the inner rendering context.
  pub fn _get_resolution(&self) -> (u32, u32) {
    let size = self.world.read_resource::<Screen>().get_size();
    (size.x.round() as u32, size.y.round() as u32)
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

  /// Restart the simulation's time. This allows animations and other time-based
  /// components to operate correctly.
  pub fn restart_time(&mut self) {
    let mut fps_counter = self.world.write_resource::<FPSCounter>();
    fps_counter.restart();
  }

  // TODO: Separate #rendering into three steps:
  // * Render the map into the pre-context, then aspect fit render to main
  // * Render debug stuff onto main
  // * Render UI onto main
  pub fn render(&mut self) {
    let mut may_ctx = self.rendering_context.take();
    if let Some(mut ctx) = may_ctx.as_mut() {
      let canvas = self
        .pre_rendering_context
        .canvas()
        .expect("pre_rendering_context has no canvas");
      let map_size = V2::new(canvas.width() as f32, canvas.height() as f32);
      self.pre_rendering_context.clear_rect(
        0.0,
        0.0,
        map_size.x as f64,
        map_size.y as f64,
      );

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

      // Aspect fit our pre_rendering_context inside the final rendering_context
      let window = ctx.canvas().expect("main rendering context has no canvas");
      let win_size = V2::new(window.width() as f32, window.height() as f32);
      let dest = AABB::aabb_to_aspect_fit_inside(map_size, win_size).round();

      ctx.set_fill_style(&"black".into());
      ctx.fill_rect(0.0, 0.0, win_size.x as f64, win_size.y as f64);
      ctx
        .draw_image_with_html_canvas_element_and_dw_and_dh(
          &canvas,
          dest.top_left.x as f64,
          dest.top_left.y as f64,
          dest.width() as f64,
          dest.height() as f64,
        )
        .expect("can't draw map");

      let viewport_to_context = |point: V2| -> V2 {
        AABB::point_inside_aspect(point, map_size, win_size)
      };

      // Draw the UI
      let _ = render::render_ui(
        &mut self.world,
        &mut self.resources,
        &mut ctx,
        viewport_to_context,
      );
      if self.debug_mode {
        let _ = render::render_ui_debug(
          &mut self.world,
          &mut self.resources,
          &mut ctx,
          viewport_to_context,
        );
      }
    } else {
      warn!("no rendering context");
    }
    self.rendering_context = may_ctx;
  }
}
