//! The Engine definition.
//!
//! The engine itself is a struct with some type variables that determine what
//! kind of rendering context and resources the engine will manage.
use super::prelude::{
    entity_local_origin, AnimationSystem, BackgroundColor, Color, DebugRenderingData, Dispatcher,
    DispatcherBuilder, FPSCounter, FenceSystem, GamepadSystem, HasRenderingContext, Join,
    MapEntity, MapRenderingData, Physics, PlayerSystem, RenderingContext, Resources, Screen,
    ScreenSystem, SystemData, TiledmapSystem, TweenSystem, World, WorldExt, ZLevel, ZoneSystem,
    AABB, V2,
};
use std::cmp::Ordering;

// TODO: Use snafu error handling


pub struct Engine<'a, 'b, Ctx, ImageResources> {
    pub base_url: String,
    pub world: World,
    pub map_rendering_context: Ctx,
    pub rendering_context: Ctx,
    pub images: ImageResources,

    dispatcher: Dispatcher<'a, 'b>,
    debug_mode: bool,
}


impl<'a, 'b, Ctx, ImageResources> Engine<'a, 'b, Ctx, ImageResources>
where
    Ctx: HasRenderingContext,
    ImageResources: Resources<<Ctx::Ctx as RenderingContext>::Image> + Default,
{
    pub fn new_with<F>(
        base_url: &str,
        dispatcher_builder: DispatcherBuilder<'a, 'b>,
        new_ctx: F,
    ) -> Self
    where
        F: Fn() -> Ctx,
    {
        let mut world = World::new();
        world.insert(BackgroundColor(Color::rgb(0, 0, 0)));

        let mut dispatcher = dispatcher_builder
            .with_thread_local(TiledmapSystem::new(base_url))
            .with_thread_local(Physics::new())
            .with_thread_local(ScreenSystem)
            .with_thread_local(AnimationSystem)
            .with_thread_local(GamepadSystem::new())
            .with_thread_local(PlayerSystem)
            .with_thread_local(TweenSystem)
            .with_thread_local(ZoneSystem)
            .with_thread_local(FenceSystem)
            //.with_thread_local(SoundSystem::new())
            //.with_thread_local(SpriteSystem)
            .build();

        dispatcher.setup(&mut world);

        // Maintain once so all our resources are created.
        world.maintain();

        Engine {
            dispatcher,
            world,
            base_url: base_url.into(),
            debug_mode: false,
            rendering_context: new_ctx(),
            map_rendering_context: new_ctx(),
            images: ImageResources::default(),
        }
    }

    pub fn set_debug_mode(&mut self, debug: bool) {
        self.debug_mode = debug;
        if debug {
            <DebugRenderingData as SystemData>::setup(&mut self.world);
        }
    }

    /// Set the width and height of the map viewport in pixels.
    /// This does not set the width and height of the window, instead it sets the
    /// width and height of the inner map viewport. The inner viewport is the
    /// one that map entities are rendered to.
    pub fn set_map_viewport_size(&mut self, w: u32, h: u32) {
        let mut screen = self.world.write_resource::<Screen>();
        screen.set_size((w, h));
        self.map_rendering_context
            .get_rendering_context()
            .set_context_size((w, h))
            .expect("could not set map rendering size");
    }


    /// Set the top left x and y position of the map viewport in map coordinates.
    pub fn set_map_viewport_top_left(&mut self, x: u32, y: u32) {
        let mut screen = self.world.write_resource::<Screen>();
        let mut viewport = screen.get_mut_viewport();
        viewport.top_left = V2::new(x as f32, y as f32);
    }


    pub fn set_window_size(&mut self, w: u32, h: u32) {
        self.rendering_context
            .get_rendering_context()
            .set_context_size((w, h))
            .expect("could not set window size");
    }

    pub fn is_debug(&self) -> bool {
        self.debug_mode
    }

    pub fn new<F: Fn() -> Ctx>(base_url: &str, new_ctx: F) -> Self {
        Self::new_with(base_url, DispatcherBuilder::new(), new_ctx)
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

    /// Find all the entities intersecting the visible map.
    fn get_map_entities(&self) -> Result<Vec<MapEntity>, String> {
        let data: MapRenderingData = self.world.system_data();
        let screen_aabb = data.screen.aabb();

        // Get all the on screen things to render.
        // Order the things by bottom to top, back to front.
        let mut ents: Vec<_> = (&data.entities, &data.positions, !&data.exiles)
            .join()
            .filter_map(|(ent, p, ())| {
                // Make sure we can see this thing (that its destination aabb intersects
                // the screen)
                let rendering = data.renderings.get(ent);
                let (w, h) = rendering.map(|r| r.size()).unwrap_or((0, 0));
                let aabb = AABB {
                    top_left: p.0,
                    extents: V2::new(w as f32, h as f32),
                };
                if !(screen_aabb.collides_with(&aabb) || aabb.collides_with(&screen_aabb)) {
                    return None;
                }

                let offset: V2 = entity_local_origin(ent, &data.shapes, &data.offsets);
                let pos = data.screen.from_map(&p.0);
                Some(MapEntity {
                    entity: ent,
                    position: pos,
                    offset,
                    rendering: rendering.cloned(),
                    z_level: data.z_levels.get(ent).cloned().unwrap_or(ZLevel(0.0)),
                })
            })
            .collect();
        ents.sort_by(|a, b| {
            if a.z_level.0 < b.z_level.0 {
                Ordering::Less
            } else if a.z_level.0 > b.z_level.0 {
                Ordering::Greater
            } else if a.position.y + a.offset.y < b.position.y + b.offset.y {
                Ordering::Less
            } else if a.position.y + a.offset.y > b.position.y + b.offset.y {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        Ok(ents)
    }

    pub fn render(&mut self) -> Result<(), String> {
        let (w, h) = self
            .map_rendering_context
            .get_rendering_context()
            .context_size()?;
        let map_size = V2::new(w as f32, h as f32);
        self.map_rendering_context.get_rendering_context().clear()?;

        let map_ents = self.get_map_entities()?;

        self.map_rendering_context
            .render_map(&mut self.world, &mut self.images, &map_ents)?;

        if self.debug_mode {
            self.map_rendering_context
                .render_map_debug(&mut self.world, &map_ents)?;
        }

        // Aspect fit our map_rendering_context inside the final rendering_context
        let win_size = self
            .rendering_context
            .get_rendering_context()
            .context_size()
            .map(|(w, h)| V2::new(w as f32, h as f32))?;
        let dest = AABB::aabb_to_aspect_fit_inside(map_size, win_size);

        self.rendering_context
            .get_rendering_context()
            .set_fill_color(&Color::rgb(0, 0, 0));
        self.rendering_context
            .get_rendering_context()
            .fill_rect(&AABB {
                top_left: V2::origin(),
                extents: win_size,
            });
        self.rendering_context
            .get_rendering_context()
            .draw_context(&self.map_rendering_context.get_rendering_context(), &dest)?;

        let viewport_to_context =
            |point: V2| -> V2 { AABB::point_inside_aspect(point, map_size, win_size) };

        // Draw the UI
        self.rendering_context.render_ui(
            &mut self.world,
            &mut self.images,
            &map_ents,
            viewport_to_context,
        )?;
        if self.debug_mode {
            self.rendering_context.render_ui_debug(
                &mut self.world,
                &map_ents,
                viewport_to_context,
            )?;
        }

        Ok(())
    }
}
