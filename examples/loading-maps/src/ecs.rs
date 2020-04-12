//! The WebEngine definition.
//!
//! TODO: Abstract engine details into a trait
//! TODO: Rename this module WebEngine that implements Engine
use log::warn;
use old_gods::{
    prelude::{
        AnimationSystem, BackgroundColor, Color, Dispatcher, DispatcherBuilder, FPSCounter,
        GamepadSystem, Physics, PlayerSystem, RenderingContext, Resources, Screen, ScreenSystem,
        SystemData, TweenSystem, World, WorldExt, AABB, V2,
    },
    rendering::standard::*,
};


pub mod systems;


pub struct ECS<'a, 'b, Ctx, Rsrc> {
    pub base_url: String,
    pub world: World,
    pub map_rendering_context: Ctx,
    pub rendering_context: Option<Ctx>,
    pub resources: Rsrc,

    dispatcher: Dispatcher<'a, 'b>,
    debug_mode: bool,
}


impl<'a, 'b, Ctx, Rsrc> ECS<'a, 'b, Ctx, Rsrc>
where
    Ctx: RenderingContext,
    Rsrc: Resources<Ctx::Image> + Default,
{
    pub fn new_with(
        base_url: &str,
        map_rendering_context: Ctx,
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
            //.with_thread_local(ZoneSystem)
            //.with_thread_local(FenceSystem)
            .build();

        dispatcher.setup(&mut world);

        // Maintain once so all our resources are created.
        world.maintain();

        ECS {
            dispatcher,
            world,
            base_url: base_url.into(),
            debug_mode: false,
            rendering_context: None,
            map_rendering_context,
            resources: Rsrc::default(),
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
        let mut screen = self.world.write_resource::<Screen>();
        screen.set_size((w, h));
        self.map_rendering_context
            .set_context_size((w, h))
            .expect("could not set map rendering size");
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

    pub fn new(base_url: &str, map_rendering_context: Ctx) -> Self {
        Self::new_with(base_url, map_rendering_context, DispatcherBuilder::new())
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

    pub fn render(&mut self) -> Result<(), String> {
        let mut may_ctx = self.rendering_context.take();
        if let Some(ctx) = may_ctx.as_mut() {
            let (w, h) = self.map_rendering_context.context_size()?;
            let map_size = V2::new(w as f32, h as f32);
            self.map_rendering_context.clear()?;

            let map_ents = get_map_entities(&mut self.world)?;

            render_map(
                &mut self.world,
                &mut self.resources,
                &mut self.map_rendering_context,
                &map_ents,
            )?;

            if self.debug_mode {
                render_map_debug(
                    &mut self.world,
                    &mut self.map_rendering_context,
                    &map_ents,
                )?;
            }

            // Aspect fit our map_rendering_context inside the final rendering_context
            let win_size = ctx
                .context_size()
                .map(|(w, h)| V2::new(w as f32, h as f32))?;
            let dest = AABB::aabb_to_aspect_fit_inside(map_size, win_size);

            ctx.set_fill_color(&Color::rgb(0, 0, 0));
            ctx.fill_rect(&AABB {
                top_left: V2::origin(),
                extents: win_size,
            });
            ctx.draw_context(&self.map_rendering_context, &dest)?;

            let viewport_to_context =
                |point: V2| -> V2 { AABB::point_inside_aspect(point, map_size, win_size) };

            // Draw the UI
            render_ui(
                &mut self.world,
                &mut self.resources,
                ctx,
                viewport_to_context,
            )?;
            if self.debug_mode {
                render_ui_debug(&mut self.world, ctx, viewport_to_context)?;
            }
        } else {
            warn!("no rendering context");
        }
        self.rendering_context = may_ctx;

        Ok(())
    }
}
