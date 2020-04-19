use super::prelude::*;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};


#[derive(SystemData)]
pub struct MapRenderingData<'s> {
    pub screen: Read<'s, Screen>,
    pub entities: Entities<'s>,
    pub positions: ReadStorage<'s, Position>,
    pub offsets: ReadStorage<'s, OriginOffset>,
    pub renderings: ReadStorage<'s, Rendering>,
    pub z_levels: ReadStorage<'s, ZLevel>,
    pub exiles: ReadStorage<'s, Exile>,
    pub shapes: ReadStorage<'s, Shape>,
}


pub struct MapEntity {
    pub entity: Entity,
    pub position: V2,
    pub offset: V2,
    pub rendering: Option<Rendering>,
    pub z_level: ZLevel,
}


#[derive(SystemData)]
pub struct DebugRenderingData<'s> {
    pub aabb_tree: Read<'s, AABBTree>,
    pub entities: Entities<'s>,
    pub global_debug_toggles: Read<'s, HashSet<RenderingToggles>>,
    pub fps: Read<'s, FPSCounter>,
    pub screen: Read<'s, Screen>,
    pub velocities: ReadStorage<'s, Velocity>,
    pub barriers: ReadStorage<'s, Barrier>,
    pub exiles: ReadStorage<'s, Exile>,
    pub players: ReadStorage<'s, Player>,
    pub positions: ReadStorage<'s, Position>,
    pub object_debug_toggles: ReadStorage<'s, ObjectRenderingToggles>,
    pub offsets: ReadStorage<'s, OriginOffset>,
    pub names: ReadStorage<'s, Name>,
    pub zones: ReadStorage<'s, Zone>,
    pub fences: ReadStorage<'s, Fence>,
    pub shapes: ReadStorage<'s, Shape>,
    pub step_fences: ReadStorage<'s, StepFence>,
    pub z_levels: ReadStorage<'s, ZLevel>,
}


/// Construct a vector of lines that form an arrow from p1 to p2
fn arrow_lines(p1: V2, p2: V2) -> Vec<V2> {
    let zero = V2::new(0.0, 0.0);
    let n = (p2 - p1).normal().unitize().unwrap_or(zero);
    let p3 = p2 - (p2 - p1).unitize().unwrap_or(zero).scalar_mul(5.0);
    let p4 = p3 + n.scalar_mul(5.0);
    let p5 = p3 - n.scalar_mul(5.0);
    vec![p1, p2, p4, p5, p2]
}

/// Construct a vector of lines that form a kind of hour glass shape.
fn point_lines(p: V2) -> Vec<V2> {
    let tl = p + V2::new(-10.0, -10.0);
    let tr = p + V2::new(10.0, -10.0);
    let bl = p + V2::new(-10., 10.0);
    let br = p + V2::new(10.0, 10.0);
    vec![tl.clone(), tr, bl, br, tl]
}


/// Defines rendering operations.
/// Given a few primitive implementations this trait provides a bunch of default
/// rendering functions. Any of these functions can be redefinied.
pub trait RenderingContext
where
    Self: Sized,
{
    type Image;
    type Font;

    fn context_size(&mut self) -> Result<(u32, u32), String>;
    fn set_context_size(&mut self, size: (u32, u32)) -> Result<(), String>;

    /// Clear the entire context.
    fn clear(&mut self) -> Result<(), String>;

    fn set_fill_color(&mut self, color: &Color);
    fn global_alpha(&mut self) -> f64;
    fn set_global_alpha(&mut self, alpha: f64);
    fn fill_rect(&mut self, aabb: &AABB);

    fn set_font(&mut self, font: &Self::Font);
    fn fill_text(&mut self, text: &str, point: &V2) -> Result<(), String>;

    fn size_of_text(&mut self, font: &Self::Font, text: &str) -> Result<(f32, f32), String>;

    fn set_stroke_color(&mut self, color: &Color);
    fn stroke_lines(&mut self, lines: &Vec<V2>);
    fn stroke_rect(&mut self, aabb: &AABB);

    fn draw_image(
        &mut self,
        img: &Self::Image,
        src: &AABB,
        destination: &AABB,
    ) -> Result<(), String>;

    /// Draw one context into another.
    fn draw_context(&mut self, context: &Self, destination: &AABB) -> Result<(), String>;

    fn font_details_to_font(&mut self, font_details: &FontDetails) -> Self::Font;


    // These remaining methods are provided by default, but may be overridden
    // by instances
}


impl RenderingContext for CanvasRenderingContext2d {
    type Image = HtmlImageElement;
    type Font = FontDetails;

    fn context_size(self: &mut CanvasRenderingContext2d) -> Result<(u32, u32), String> {
        let canvas = self
            .canvas()
            .ok_or("rendering context has no canvas".to_string())?;
        Ok((canvas.width(), canvas.height()))
    }

    fn set_context_size(
        self: &mut CanvasRenderingContext2d,
        (w, h): (u32, u32),
    ) -> Result<(), String> {
        let canvas = self
            .canvas()
            .ok_or("rendering context has no canvas".to_string())?;
        canvas.set_width(w);
        canvas.set_height(h);
        Ok(())
    }

    fn clear(&mut self) -> Result<(), String> {
        self.set_fill_style(&JsValue::from(&Color::rgba(0, 0, 0, 0)));
        let (w, h) = self.context_size()?;
        self.fill_rect(&AABB::new(0.0, 0.0, w as f32, h as f32));
        Ok(())
    }

    fn set_fill_color(self: &mut CanvasRenderingContext2d, color: &Color) {
        let alpha = self.global_alpha();
        self.set_global_alpha(color.a as f64 / 255.0);
        CanvasRenderingContext2d::set_fill_style(self, &JsValue::from(color));
        self.set_global_alpha(alpha);
    }

    fn global_alpha(self: &mut CanvasRenderingContext2d) -> f64 {
        CanvasRenderingContext2d::global_alpha(self)
    }

    fn set_global_alpha(self: &mut CanvasRenderingContext2d, alpha: f64) {
        CanvasRenderingContext2d::set_global_alpha(self, alpha);
    }

    fn fill_rect(self: &mut CanvasRenderingContext2d, aabb: &AABB) {
        CanvasRenderingContext2d::fill_rect(
            self,
            aabb.top_left.x as f64,
            aabb.top_left.y as f64,
            aabb.extents.x as f64,
            aabb.extents.y as f64,
        );
    }

    // TODO: Better web font handling
    // https://developer.mozilla.org/en-US/docs/Web/API/FontFace
    fn set_font(&mut self, font: &Self::Font) {
        CanvasRenderingContext2d::set_font(self, &font.to_css_string());
    }

    fn fill_text(&mut self, text: &str, point: &V2) -> Result<(), String> {
        CanvasRenderingContext2d::fill_text(self, text, point.x as f64, point.y as f64)
            .map_err(|e| format!("cannot fill text: {:#?}", e))
    }

    /// This isn't working very well.
    /// TODO: Better web text measurement.
    fn size_of_text(&mut self, font: &Self::Font, text: &str) -> Result<(f32, f32), String> {
        self.set_font(font);
        let num_lines = text.lines().count();
        let height = font.size * num_lines as u16;
        let metrics = CanvasRenderingContext2d::measure_text(self, &text)
            .map_err(|e| format!("cannot measure text: {:#?}", e))?;
        let width = metrics.width();
        Ok((width as f32, height as f32))
    }

    fn set_stroke_color(&mut self, color: &Color) {
        let alpha = self.global_alpha();
        self.set_global_alpha(color.a as f64 / 255.0);
        CanvasRenderingContext2d::set_stroke_style(self, &JsValue::from(color));
        self.set_global_alpha(alpha);
    }

    fn stroke_lines(&mut self, lines: &Vec<V2>) {
        self.begin_path();
        let mut iter = lines.iter();
        iter.next()
            .iter()
            .for_each(|point| self.move_to(point.x as f64, point.y as f64));
        iter.for_each(|point| self.line_to(point.x as f64, point.y as f64));
        self.close_path();
        self.stroke();
    }

    fn stroke_rect(&mut self, aabb: &AABB) {
        CanvasRenderingContext2d::stroke_rect(
            self,
            aabb.top_left.x as f64,
            aabb.top_left.y as f64,
            aabb.extents.x as f64,
            aabb.extents.y as f64,
        );
    }

    fn draw_image(
        &mut self,
        img: &Self::Image,
        src: &AABB,
        destination: &AABB,
    ) -> Result<(), String> {
        let res = self
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                img,
                src.top_left.x as f64,
                src.top_left.y as f64,
                src.extents.x as f64,
                src.extents.y as f64,
                destination.top_left.x as f64,
                destination.top_left.y as f64,
                destination.extents.x as f64,
                destination.extents.y as f64,
            );
        res.map_err(|e| format!("error drawing image: {:#?}", e))
    }

    fn draw_context(&mut self, context: &Self, dest: &AABB) -> Result<(), String> {
        self.draw_image_with_html_canvas_element_and_dw_and_dh(
            &context
                .canvas()
                .ok_or("can't draw map to window".to_string())?,
            dest.top_left.x as f64,
            dest.top_left.y as f64,
            dest.width() as f64,
            dest.height() as f64,
        )
        .map_err(|e| format!("can't draw context: {:#?}", e))
    }

    fn font_details_to_font(&mut self, font_details: &FontDetails) -> Self::Font {
        font_details.clone()
    }
}


pub trait HasRenderingContext
where
    Self: Sized,
{
    type Ctx: RenderingContext;

    fn get_rendering_context(&mut self) -> &mut Self::Ctx;

    fn context_size(&mut self) -> Result<(u32, u32), String> {
        self.get_rendering_context().context_size()
    }

    fn set_context_size(&mut self, size: (u32, u32)) -> Result<(), String> {
        self.get_rendering_context().set_context_size(size)
    }


    /// Clear the entire context.
    fn clear(&mut self) -> Result<(), String> {
        self.get_rendering_context().clear()
    }


    fn set_fill_color(&mut self, color: &Color) {
        self.get_rendering_context().set_fill_color(color);
    }

    fn global_alpha(&mut self) -> f64 {
        self.get_rendering_context().global_alpha()
    }

    fn set_global_alpha(&mut self, alpha: f64) {
        self.get_rendering_context().set_global_alpha(alpha);
    }

    fn fill_rect(&mut self, aabb: &AABB) {
        self.get_rendering_context().fill_rect(aabb);
    }


    fn set_font(&mut self, font: &<Self::Ctx as RenderingContext>::Font) {
        self.get_rendering_context().set_font(font);
    }

    fn fill_text(&mut self, text: &str, point: &V2) -> Result<(), String> {
        self.get_rendering_context().fill_text(text, point)
    }


    fn size_of_text(
        &mut self,
        font: &<Self::Ctx as RenderingContext>::Font,
        text: &str,
    ) -> Result<(f32, f32), String> {
        self.get_rendering_context().size_of_text(font, text)
    }


    fn set_stroke_color(&mut self, color: &Color) {
        self.get_rendering_context().set_stroke_color(color);
    }

    fn stroke_lines(&mut self, lines: &Vec<V2>) {
        self.get_rendering_context().stroke_lines(lines);
    }

    fn stroke_rect(&mut self, aabb: &AABB) {
        self.get_rendering_context().stroke_rect(aabb);
    }


    fn draw_image(
        &mut self,
        img: &<Self::Ctx as RenderingContext>::Image,
        src: &AABB,
        dest: &AABB,
    ) -> Result<(), String> {
        self.get_rendering_context().draw_image(img, src, dest)
    }

    /// Draw one context into another.
    fn draw_context(&mut self, context: &Self::Ctx, destination: &AABB) -> Result<(), String> {
        self.get_rendering_context()
            .draw_context(context, destination)
    }


    fn font_details_to_font(
        &mut self,
        font_details: &FontDetails,
    ) -> <Self::Ctx as RenderingContext>::Font {
        self.get_rendering_context()
            .font_details_to_font(font_details)
    }


    fn draw_text(&mut self, text: &Text, pos: &V2) -> Result<(), String> {
        let ctx = self.get_rendering_context();
        let font = ctx.font_details_to_font(&text.font);
        ctx.set_font(&font);
        ctx.set_fill_color(&text.color);
        ctx.fill_text(text.text.as_str(), pos)
    }

    fn draw_sprite(
        &mut self,
        src: AABB,
        destination: AABB,
        flip_horizontal: bool,
        flip_vertical: bool,
        flip_diagonal: bool,
        tex: &<Self::Ctx as RenderingContext>::Image,
    ) -> Result<(), String> {
        //let mut should_flip_horizontal = false;
        //let should_flip_vertical;
        //let mut angle = 0.0;

        match (flip_diagonal, flip_horizontal, flip_vertical) {
            // TODO: Support CanvasRenderingContext2d flipped tiles
            //(true, true, true) => {
            //  angle = -90.0;
            //  should_flip_vertical = true;
            //},
            //(true, a, b) => {
            //  angle = -90.0;
            //  should_flip_vertical = !b;
            //  should_flip_horizontal = a;
            //}
            //(false, a, b) => {
            //  should_flip_horizontal = a;
            //  should_flip_vertical = b;
            //}
            _ => {}
        }
        self.get_rendering_context()
            .draw_image(tex, &src, &destination)
    }

    fn draw_rendering<T>(
        &mut self,
        rsc: &mut T,
        point: &V2,
        rendering: &Rendering,
    ) -> Result<(), String>
    where
        T: Resources<<Self::Ctx as RenderingContext>::Image>,
    {
        match &rendering.primitive {
            RenderingPrimitive::TextureFrame(f) => {
                let res = rsc.when_loaded(&f.sprite_sheet, |tex| {
                    let dest = AABB::new(point.x, point.y, f.size.0 as f32, f.size.1 as f32);
                    let src = AABB::new(
                        f.source_aabb.x as f32,
                        f.source_aabb.y as f32,
                        f.source_aabb.w as f32,
                        f.source_aabb.h as f32,
                    );
                    let alpha = self.get_rendering_context().global_alpha();
                    self.get_rendering_context()
                        .set_global_alpha(rendering.alpha as f64 / 255.0);
                    self.draw_sprite(
                        src,
                        dest,
                        f.is_flipped_horizontally,
                        f.is_flipped_vertically,
                        f.is_flipped_diagonally,
                        &tex,
                    )?;
                    self.get_rendering_context().set_global_alpha(alpha);
                    Ok(())
                });
                match res {
                    Err(msg) => Err(msg),           // error loading
                    Ok(None) => Ok(()),             // still loading
                    Ok(Some(Ok(()))) => Ok(()),     // drew fine
                    Ok(Some(Err(msg))) => Err(msg), // drawing error!
                }
            }

            RenderingPrimitive::Text(t) => self.draw_text(t, point),
        }
    }

    /// TODO: Change this to return V2
    fn measure_text(&mut self, text: &Text) -> Result<(f32, f32), String> {
        let ctx = self.get_rendering_context();
        let font = ctx.font_details_to_font(&text.font);
        ctx.size_of_text(&font, &text.text)
    }

    fn draw_map_aabb(&mut self, screen: &Screen) {
        let size = screen.get_size();
        self.get_rendering_context().stroke_rect(&AABB::new(
            0.0,
            0.0,
            size.x as f32,
            size.y as f32,
        ));
    }


    fn draw_velocity(
        &mut self,
        data: &DebugRenderingData,
        map_ent: &MapEntity,
        viewport_to_context: impl Fn(V2) -> V2,
    ) {
        let velo = if let Some(velo) = data.velocities.get(map_ent.entity) {
            velo
        } else {
            return;
        };

        let v = if velo.0.magnitude() < 1e-10 {
            return;
        } else {
            velo.0
        };
        let offset: V2 = entity_local_origin(map_ent.entity, &data.shapes, &data.offsets);
        let p1 = map_ent.position + offset;
        let p2 = p1 + v;
        let lines = arrow_lines(viewport_to_context(p1), viewport_to_context(p2));
        self.get_rendering_context()
            .set_stroke_color(&Color::rgb(255, 255, 0));
        self.get_rendering_context().stroke_lines(&lines);
    }


    fn draw_aabb_tree(
        &mut self,
        data: &DebugRenderingData,
        player: &Option<(&Player, &ZLevel)>,
        from_viewport: impl Fn(V2) -> V2,
    ) -> Result<(), String> {
        let mbrs = data
            .aabb_tree
            .rtree
            .lookup_in_rectangle(&data.screen.aabb().to_mbr());
        for EntityBounds {
            bounds: mbr,
            entity_id: id,
        } in mbrs
        {
            let entity = data.entities.entity(*id);
            let z = data
                .z_levels
                .get(entity)
                .or(player.map(|p| p.1))
                .cloned()
                .unwrap_or(ZLevel(0.0));
            let alpha = if player.is_some() {
                if z.0 == (player.unwrap().1).0 {
                    255
                } else {
                    50
                }
            } else {
                255
            };
            let color = if data.exiles.contains(entity) {
                Color::rgba(255, 0, 255, alpha)
            } else {
                Color::rgba(255, 255, 0, alpha)
            };
            let aabb = AABB::from_mbr(&mbr);
            let aabb = AABB::from_points(
                from_viewport(data.screen.from_map(&aabb.top_left)),
                from_viewport(data.screen.from_map(&aabb.upper())),
            );

            self.set_stroke_color(&color);
            self.set_fill_color(&color);
            self.stroke_rect(&aabb);
            if let Some(name) = data.names.get(entity) {
                let p = V2::new(aabb.top_left.x, aabb.bottom());
                let mut text = Self::debug_text(name.0.as_str());
                text.color = color;
                self.draw_text(&text, &p)?;
            }
        }

        Ok(())
    }


    fn draw_zone(
        &mut self,
        data: &DebugRenderingData,
        map_ent: &MapEntity,
        viewport_to_context: impl Fn(V2) -> V2,
    ) -> Result<(), String> {
        if let Some(_zone) = data.zones.get(map_ent.entity) {
            if let Some(shape) = data.shapes.get(map_ent.entity) {
                let mut color = Color::rgb(139, 175, 214);
                let alpha = if data.exiles.contains(map_ent.entity) {
                    128
                } else {
                    255
                };
                color.a = alpha;
                self.get_rendering_context().set_fill_color(&color);

                let extents = shape.extents();
                let aabb = AABB::from_points(
                    viewport_to_context(data.screen.from_map(&map_ent.position)),
                    viewport_to_context(data.screen.from_map(&(map_ent.position + extents))),
                );
                self.get_rendering_context().fill_rect(&aabb);

                if let Some(name) = data.names.get(map_ent.entity) {
                    let p = V2::new(aabb.top_left.x, aabb.bottom());
                    let mut text = Self::debug_text(name.0.as_str());
                    text.color = color;
                    self.draw_text(&text, &p)?;
                }
            }
        }
        Ok(())
    }


    fn draw_fence(
        &mut self,
        data: &DebugRenderingData,
        map_ent: &MapEntity,
        viewport_to_context: impl Fn(V2) -> V2,
    ) -> Result<(), String> {
        let mut fences = vec![];
        if let Some(fence) = data.fences.get(map_ent.entity) {
            fences.push((fence, Color::rgb(153, 102, 255)));
        }
        if let Some(step_fence) = data.step_fences.get(map_ent.entity) {
            fences.push((&step_fence.0, Color::rgb(102, 0, 255)));
        }

        for (fence, color) in fences {
            let pos = data.screen.from_map(&map_ent.position);
            let lines: Vec<V2> = fence
                .points
                .iter()
                .map(|p| viewport_to_context(pos + *p))
                .collect();
            self.get_rendering_context().set_fill_color(&color);
            self.get_rendering_context().stroke_lines(&lines);
            if let Some(name) = data.names.get(map_ent.entity) {
                let text = Self::debug_text(name.0.as_str());
                self.draw_text(&text, &pos)?;
            }
        }

        Ok(())
    }

    fn draw_player(
        &mut self,
        data: &DebugRenderingData,
        map_ent: &MapEntity,
        viewport_to_context: impl Fn(V2) -> V2,
    ) {
        let p = viewport_to_context(data.screen.from_map(&(map_ent.position + map_ent.offset)));
        self.get_rendering_context()
            .set_fill_color(&Color::rgb(0, 255, 255));
        self.get_rendering_context()
            .fill_rect(&AABB::new(p.x - 24.0, p.y - 24.0, 48.0, 48.0));
        //let text =
        //  Self::debug_text(format!("{:?}", player));
        //RenderText::draw_text(canvas, resources, &p);
    }

    fn draw_screen(&mut self, data: &DebugRenderingData) {
        let screen_aabb = data.screen.aabb();
        let window_aabb = AABB::from_points(
            data.screen.from_map(&screen_aabb.lower()),
            data.screen.from_map(&screen_aabb.upper()),
        );
        self.get_rendering_context()
            .set_stroke_color(&Color::rgb(0, 255, 0));
        self.get_rendering_context().stroke_rect(&window_aabb);

        let focus_aabb = data.screen.focus_aabb();
        let window_focus_aabb = AABB::from_points(
            data.screen.from_map(&focus_aabb.top_left),
            data.screen.from_map(&focus_aabb.lower()),
        );
        self.get_rendering_context().stroke_rect(&window_focus_aabb);
    }


    fn draw_action(
        &mut self,
        data: &DebugRenderingData,
        map_ent: &MapEntity,
        viewport_to_context: impl Fn(V2) -> V2,
    ) {
        let is_exiled = data.exiles.contains(map_ent.entity);

        let color = if is_exiled {
            Color::rgb(255, 255, 255)
        } else {
            Color::rgb(252, 240, 5)
        };
        self.get_rendering_context().set_fill_color(&color);

        let a = viewport_to_context(data.screen.from_map(&map_ent.position));
        let b = a + V2::new(10.0, -20.0);
        let c = a + V2::new(-10.0, -20.0);
        let lines = vec![a, b, c, a];
        self.get_rendering_context().stroke_lines(&lines);
    }


    fn draw_shape(
        &mut self,
        data: &DebugRenderingData,
        map_ent: &MapEntity,
        viewport_to_context: impl Fn(V2) -> V2,
    ) -> Option<()> {
        let color = Color::rgb(128, 128, 255);
        self.get_rendering_context().set_fill_color(&color);

        let shape = data.shapes.get(map_ent.entity)?;
        let lines: Vec<V2> = shape
            .vertices_closed()
            .into_iter()
            .map(|v| viewport_to_context(data.screen.from_map(&(map_ent.position + v))))
            .collect();
        self.get_rendering_context().stroke_lines(&lines);

        Some(())
    }


    fn draw_barrier(
        &mut self,
        data: &DebugRenderingData,
        show_collision_info: bool,
        player_z: f32,
        map_ent: &MapEntity,
        viewport_to_context: impl Fn(V2) -> V2,
    ) -> Option<()> {
        let _barrier = data.barriers.get(map_ent.entity)?;
        let shape = data.shapes.get(map_ent.entity)?;
        let z = data.z_levels.get(map_ent.entity)?;
        let is_exiled = data
            .exiles
            .get(map_ent.entity)
            .map(|_| true)
            .unwrap_or(false);
        let alpha = if z.0 == player_z { 255 } else { 50 };
        let color = if is_exiled {
            Color::rgba(255, 255, 255, alpha)
        } else {
            Color::rgba(255, 0, 0, alpha)
        };
        self.get_rendering_context().set_stroke_color(&color);

        let lines: Vec<V2> = shape
            .vertices_closed()
            .into_iter()
            .map(|v| viewport_to_context(map_ent.position + v))
            .collect();
        self.get_rendering_context().stroke_lines(&lines);

        if show_collision_info {
            // Draw the potential separating axes
            let axes = shape.potential_separating_axes();
            let midpoints = shape.midpoints();
            // light red
            let color = Color::rgb(255, 128, 128);
            self.get_rendering_context().set_stroke_color(&color);
            for (axis, midpoint) in axes.into_iter().zip(midpoints) {
                let midpoint = viewport_to_context(midpoint + map_ent.position);
                let lines = arrow_lines(midpoint, midpoint + (axis.scalar_mul(20.0)));
                self.get_rendering_context().stroke_lines(&lines);
            }

            // Draw its collision with other shapes
            let pos = data.positions.get(map_ent.entity).unwrap();
            let aabb = shape.aabb().translate(&pos.0);
            let broad_phase_collisions: Vec<(Entity, AABB)> =
                data.aabb_tree.query(&data.entities, &aabb, &map_ent.entity);
            broad_phase_collisions
                .into_iter()
                .for_each(|(other_ent, other_aabb)| {
                    // Draw the union of their aabbs to show the
                    // broad phase collision
                    let color = Color::rgb(255, 128, 64); // orange
                    self.get_rendering_context().set_stroke_color(&color);
                    let both_aabb = AABB::union(&aabb, &other_aabb);
                    self.stroke_rect(&AABB::from_points(
                        viewport_to_context(data.screen.from_map(&both_aabb.lower())),
                        viewport_to_context(data.screen.from_map(&both_aabb.upper())),
                    ));

                    // Find out if they actually collide and what the
                    // mtv is
                    let other_shape = if let Some(other_shape) = data.shapes.get(other_ent) {
                        other_shape
                    } else {
                        return;
                    };
                    let other_position = data.positions.get(other_ent);
                    if other_position.is_none() {
                        // This is probably an item that's in an inventory.
                        return;
                    }
                    let other_position = other_position.unwrap();
                    let mtv = shape.mtv_apart(pos.0, &other_shape, other_position.0);
                    mtv.map(|mtv| {
                        self.get_rendering_context()
                            .set_stroke_color(&Color::rgb(255, 255, 255));

                        let from = viewport_to_context(data.screen.from_map(&other_aabb.center()));
                        let to = viewport_to_context(data.screen.from_map(&(other_aabb.center() + mtv)));
                        let lines = point_lines(from);
                        self.get_rendering_context().stroke_lines(&lines);

                        let lines = arrow_lines(from, to);
                        self.get_rendering_context().stroke_lines(&lines);
                    });
                });
        }

        Some(())
    }


    fn draw_position(
        &mut self,
        data: &DebugRenderingData,
        map_ent: &MapEntity,
        viewport_to_context: impl Fn(V2) -> V2,
    ) -> Result<(), String> {
        self.get_rendering_context()
            .set_stroke_color(&Color::rgb(0, 0, 255));

        let draw = |label: &str, v: V2, ctx: &mut Self| -> Result<(), String> {
            let v = viewport_to_context(v);
            ctx.get_rendering_context().stroke_rect(&AABB::new(
                v.x as f32 - 2.0,
                v.y as f32 - 2.0,
                4.0,
                4.0,
            ));

            let pos_str = format!("{}: ({:.1}, {:.1})", label, v.x, v.y,);
            let text = Self::debug_map_text(&pos_str);
            ctx.get_rendering_context()
                .set_fill_color(&Color::rgb(255, 255, 255));
            ctx.draw_text(&text, &v)?;
            Ok(())
        };

        let name = data.names.get(map_ent.entity).map(|Name(n)| n.as_str());
        let pos = "pos";
        let position_label = &name.unwrap_or(pos);
        draw(position_label, map_ent.position, self)?;

        if map_ent.offset != V2::origin() {
            self.get_rendering_context()
                .set_stroke_color(&Color::rgb(0, 200, 200));
            self.get_rendering_context().stroke_lines(&arrow_lines(
                viewport_to_context(map_ent.position),
                viewport_to_context(map_ent.position + map_ent.offset),
            ));
            draw("orgo", map_ent.position + map_ent.offset, self)?;
        }

        Ok(())
    }


    /// Render the map in a standard way, compositing all of the renderings from back to front, bottom to top.
    /// Returns the drawn entities.
    fn render_map<Rsrc>(
        &mut self,
        world: &mut World,
        resources: &mut Rsrc,
        map_entities: &Vec<MapEntity>,
    ) -> Result<(), String>
    where
        Rsrc: Resources<<Self::Ctx as RenderingContext>::Image>,
    {
        let background_color: Read<BackgroundColor> = world.system_data();
        let size = self.get_rendering_context().context_size()?;
        // Render into our render target texture
        self.get_rendering_context()
            .set_fill_color(&background_color.0);
        self.get_rendering_context().fill_rect(&AABB {
            top_left: V2::new(0.0, 0.0),
            extents: V2::new(size.0 as f32, size.1 as f32),
        });
        // Draw the map renderings
        for map_ent in map_entities.iter() {
            if let Some(rendering) = &map_ent.rendering {
                self.draw_rendering(resources, &map_ent.position, &rendering)?;
            }
        }

        Ok(())
    }

    fn render_map_entity_debug(
        &mut self,
        data: &DebugRenderingData,
        toggles: &HashSet<&RenderingToggles>,
        player: Option<(&Player, &ZLevel)>,
        map_ent: &MapEntity,
        viewport_to_context: impl Fn(V2) -> V2,
    ) -> Result<(), String> {
        if toggles.contains(&RenderingToggles::Positions) {
            self.draw_position(data, map_ent, &viewport_to_context)?;
        }

        if toggles.contains(&RenderingToggles::Velocities) {
            self.draw_velocity(data, map_ent, &viewport_to_context);
        }

        if toggles.contains(&RenderingToggles::Zones) {
            self.draw_zone(data, map_ent, &viewport_to_context)?;
        }

        if toggles.contains(&RenderingToggles::Fences) {
            self.draw_fence(data, map_ent, &viewport_to_context)?;
        }

        if toggles.contains(&RenderingToggles::Players)
            && !toggles.contains(&RenderingToggles::Barriers)
        {
            self.draw_player(data, map_ent, &viewport_to_context);
        }

        if toggles.contains(&RenderingToggles::Actions) {
            self.draw_action(data, map_ent, &viewport_to_context);
        }

        if toggles.contains(&RenderingToggles::Shapes) {
            self.draw_shape(data, map_ent, &viewport_to_context);
        }

        let show_collision_info = toggles.contains(&RenderingToggles::CollisionInfo);
        if toggles.contains(&RenderingToggles::Barriers) || show_collision_info {
            let player_z = player.map(|(_, z)| z.0).unwrap_or(0.0);
            self.draw_barrier(
                data,
                show_collision_info,
                player_z,
                map_ent,
                viewport_to_context,
            );
        }

        Ok(())
    }

    fn render_map_debug(
        &mut self,
        _world: &mut World,
        _map_entities: &Vec<MapEntity>,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Renders the user interface.
    fn render_ui<Rsrc: Resources<<Self::Ctx as RenderingContext>::Image>, F>(
        &mut self,
        _world: &mut World,
        _resources: &mut Rsrc,
        _map_entities: &Vec<MapEntity>,
        // The function needed to convert a point in the map viewport to the context.
        _viewport_to_context: F,
    ) -> Result<(), String>
    where
        F: Fn(V2) -> V2,
    {
        //self.render_actions(world, viewport_to_context)?;
        Ok(())
    }


    /// Renders debugging info for the user interface.
    fn render_ui_debug(
        &mut self,
        world: &mut World,
        map_entities: &Vec<MapEntity>,
        // The function needed to convert a point in the map viewport to the context.
        viewport_to_context: impl Fn(V2) -> V2,
    ) -> Result<(), String> {
        let data: DebugRenderingData = world.system_data();
        let next_rect = if data.global_debug_toggles.contains(&RenderingToggles::FPS) {
            let fps_text = Self::debug_text(&data.fps.current_fps_string());
            let size = self.measure_text(&fps_text)?;
            let pos = V2::new(0.0, size.1);
            self.get_rendering_context()
                .set_fill_color(&Color::rgb(255, 255, 255));
            self.draw_text(&fps_text, &pos)?;

            // Draw a graph of the FPS
            {
                let averages = data.fps.second_averages();
                let max_average = averages.iter().fold(0.0, |a, b| f32::max(a, *b));
                let mut x = pos.x + size.0 + 2.0;
                let height = size.1;
                let y = (pos.y + height).round();
                // TODO: Fix the drawing of the FPS graph
                let mut points = vec![
                    V2::new(x + FPS_COUNTER_BUFFER_SIZE as f32, y),
                    V2::new(x, y),
                ];
                for avg in averages.into_iter() {
                    let percent = avg / max_average;
                    points.push(V2::new(x, y - (percent * height)));
                    x += 1.0
                }
                self.get_rendering_context()
                    .set_stroke_color(&super::color::css::gold());
                self.get_rendering_context().stroke_lines(&points);
            }

            AABB {
                top_left: pos,
                extents: V2 {
                    x: size.0,
                    y: size.1,
                },
            }
        } else {
            AABB::identity()
        };

        let toggles = &data.global_debug_toggles;
        if toggles.contains(&RenderingToggles::EntityCount) {
            let count: u32 = (&data.entities).join().fold(0, |n, _| n + 1);
            let mut text = Self::debug_text(format!("Entities: {}", count).as_str());
            text.color = Color::rgb(0, 255, 100);
            let pos = V2::new(0.0, next_rect.bottom() as f32 + 10.0);
            self.draw_text(&text, &pos)?;
        }

        if toggles.contains(&RenderingToggles::AABBTree) {
            let player = (&data.players, &data.z_levels)
                .join()
                .filter(|(p, _)| p.0 == 0)
                .collect::<Vec<_>>()
                .first()
                .cloned();
            self.draw_aabb_tree(&data, &player, &viewport_to_context)?;
        }

        if toggles.contains(&RenderingToggles::Screen) {
            self.draw_screen(&data);
        }

        let player = (&data.players, &data.z_levels)
            .join()
            .filter(|(p, _)| p.0 == 0)
            .collect::<Vec<_>>()
            .first()
            .cloned();
        let empty_toggles = HashSet::new();
        for map_ent in map_entities.into_iter() {
            let obj_toggles: &HashSet<_> = data
                .object_debug_toggles
                .get(map_ent.entity)
                .map(|t| &t.0)
                .unwrap_or(&empty_toggles);
            let toggles: &HashSet<_> = &toggles.union(&obj_toggles).collect::<HashSet<_>>();
            self.render_map_entity_debug(&data, toggles, player, map_ent, &viewport_to_context)?;
        }


        Ok(())
    }

    fn fancy_font() -> FontDetails {
        FontDetails {
            path: "Georgia,Times,Times New Roman,serif".to_string(),
            size: 18,
        }
    }

    fn fancy_text(msg: &str) -> Text {
        Text {
            text: msg.to_string(),
            font: Self::fancy_font(),
            color: Color::rgb(255, 255, 255),
            size: (16, 16),
        }
    }

    fn normal_font() -> FontDetails {
        FontDetails {
            path: "Futura,Trebuchet MS,Arial,sans-serif".to_string(),
            size: 16,
        }
    }

    fn normal_text(msg: &str) -> Text {
        Text {
            text: msg.to_string(),
            font: Self::normal_font(),
            color: Color::rgb(255, 255, 255),
            size: (16, 16),
        }
    }

    fn debug_font_details() -> FontDetails {
        FontDetails {
            path: "Consolas,monaco,monospace".to_string(),
            size: 16,
        }
    }

    fn debug_text(text: &str) -> Text {
        Text {
            text: text.to_string(),
            font: Self::debug_font_details(),
            color: Color::rgb(255, 255, 255),
            size: (16, 16),
        }
    }

    fn debug_map_text(text: &str) -> Text {
        Text {
            text: text.to_string(),
            font: Self::debug_font_details(),
            color: Color::rgb(255, 255, 255),
            size: (12, 12),
        }
    }
}


/// When you want to call the default implementations of a rendering context you
/// can wrap it in a DefaultRenderingContext.
pub struct DefaultRenderingContext<T> {
    pub context: T,
}


impl HasRenderingContext for DefaultRenderingContext<CanvasRenderingContext2d> {
    type Ctx = CanvasRenderingContext2d;

    fn get_rendering_context(&mut self) -> &mut CanvasRenderingContext2d {
        &mut self.context
    }
}
