use super::{components::inventory::Inventory, systems::looting::Loot};
use log::{trace, warn};
use old_gods::{
    color::css,
    prelude::{
        Color, DefaultRenderingContext, Exile, Join, Name, Player, Position, Read, ReadStorage,
        Resources, World, AABB, V2,
    },
    rendering::*,
};
use std::ops::{Deref, DerefMut};
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

mod inventory;


pub struct WebRenderingContext(pub DefaultRenderingContext<CanvasRenderingContext2d>);


impl WebRenderingContext {
    pub fn new() -> Self {
        let context = window()
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
        WebRenderingContext(DefaultRenderingContext { context })
    }

    pub fn canvas(&self) -> Option<HtmlCanvasElement> {
        self.0.context.canvas()
    }
}


impl Deref for WebRenderingContext {
    type Target = DefaultRenderingContext<CanvasRenderingContext2d>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl DerefMut for WebRenderingContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl HasRenderingContext for WebRenderingContext {
    type Ctx = CanvasRenderingContext2d;

    fn get_rendering_context(&mut self) -> &mut Self::Ctx {
        &mut self.0.context
    }

    fn render_ui<R, F>(
        &mut self,
        world: &mut World,
        resources: &mut R,
        viewport_to_context: F,
    ) -> Result<(), String>
    where
        F: Fn(V2) -> V2,
        R: Resources<<Self::Ctx as RenderingContext>::Image>,
    {
        self.deref_mut()
            .render_ui(world, resources, viewport_to_context)?;

        self.set_fill_color(&css::pink());
        self.fill_rect(&AABB {
            top_left: V2::new(10.0, 10.0),
            extents: V2::new(100.0, 100.0),
        });


        let (exiles, inventories, loots, names, positions, players): (
            ReadStorage<Exile>,
            ReadStorage<Inventory>,
            Read<Vec<Loot>>,
            ReadStorage<Name>,
            ReadStorage<Position>,
            ReadStorage<Player>,
        ) = world.system_data();

        let (ctx_w, ctx_h) = self.context_size()?;
        let center = V2::new(ctx_w as f32, ctx_h as f32).scalar_mul(0.5);
        let slot_size = V2::new(48.0, 48.0);
        let slot_padding = 2.0;
        let frame_padding = 6.0;
        let total_slot_size = slot_size + V2::new(slot_padding * 2.0, slot_padding * 2.0);
        let starting_point = V2::new(center.x - (Loot::COLS as f32 * total_slot_size.x), frame_padding);

        // Draw the first looting
        if let Some(loot) = loots.first() {
            if let Some(inventory) = inventories.get(loot.ent_of_inventory_here) {
                let name = names
                    .get(loot.ent_of_inventory_here)
                    .cloned()
                    .unwrap_or(Name("unknown".into()));
                for (item, ndx) in inventory.items.iter().zip(0..) {
                    let x_ndx = ndx % Loot::COLS as i32;
                    let y_ndx = ndx / Loot::COLS as i32;
                    let point =
                        starting_point + V2::new(x_ndx as f32, y_ndx as f32) * total_slot_size;
                    self.set_fill_color(&css::dark_slate_gray());
                    self.fill_rect(&AABB {
                        top_left: point - V2::new(slot_padding, slot_padding),
                        extents: total_slot_size,
                    });

                    self.draw_rendering(resources, &point, &item.rendering)?;

                    self.set_stroke_color(&Color::rgb(127, 127, 40));
                    let (w, h) = item.rendering.size();
                    self.stroke_rect(&AABB {
                        top_left: point,
                        extents: V2::new(w as f32, h as f32),
                    });
                }
            } else {
                warn!("looting no inventory");
            }
        }
        Ok(())
    }
}
