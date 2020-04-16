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


        let (inventories, loots, names): (
            ReadStorage<Inventory>,
            Read<Vec<Loot>>,
            ReadStorage<Name>,
        ) = world.system_data();

        let (ctx_w, ctx_h) = self.context_size()?;
        let center = V2::new(ctx_w as f32, ctx_h as f32).scalar_mul(0.5);
        let slot_size = V2::new(48.0, 48.0);
        let slot_padding = 2.0;
        let frame_padding = 6.0;
        let total_slot_size = slot_size + V2::new(slot_padding * 2.0, slot_padding * 2.0);

        // Draw the first looting
        if let Some(loot) = loots.first() {
            if let Some(inventory) = inventories.get(loot.ent_of_inventory_here) {
                let name = names
                    .get(loot.ent_of_inventory_here)
                    .cloned()
                    .unwrap_or(Name("unknown".into()));
                // The left edge of the entire inv frame
                let frame_left =
                    center.x - (Loot::COLS as f32 * total_slot_size.x) - frame_padding * 2.0;
                // The top edge of the entire inv frame
                let frame_top = 0.0;
                // Create and measure the title
                let title = format!(
                    "{}'s inventory{}",
                    name.0,
                    if inventory.item_len() == 0 {
                        " is empty!"
                    } else {
                        ""
                    }
                );
                let title = Self::fancy_text(&title);
                let title_size = self.measure_text(&title)?;
                let title_point = V2::new(frame_left + frame_padding, frame_top + frame_padding);
                // Now determine the starting point for the items
                let num_item_rows = (inventory.item_len() as f32 / Loot::COLS as f32).ceil();
                let num_item_rows = f32::max(1.0, num_item_rows);
                let first_item_point =
                    V2::new(title_point.x, title_point.y + title_size.1 + frame_padding);
                let total_items_size = V2::new(
                    Loot::COLS as f32 * total_slot_size.x,
                    num_item_rows * total_slot_size.y,
                );
                // Next we can determine the inventory AABB.
                let inv_bg_size = V2::new(
                    frame_padding + total_items_size.x + frame_padding,
                    frame_padding
                        + title_size.1
                        + frame_padding
                        + total_items_size.y
                        + frame_padding,
                );
                let inv_bg_color = Color::rgb(0x33, 0x33, 0x33);
                let inv_bg_aabb = AABB {
                    top_left: V2::new(frame_left, frame_top),
                    extents: inv_bg_size,
                };
                // Finally we can draw the inventory frame
                self.set_fill_color(&inv_bg_color);
                self.set_stroke_color(&inv_bg_color);
                self.fill_rect(&inv_bg_aabb);
                self.stroke_rect(&inv_bg_aabb);
                // Draw the title
                self.draw_text(&title, &(title_point + V2::new(0.0, title_size.1)))?;

                let dark_color = css::dark_slate_gray();
                let light_color = Color::rgb(127, 127, 40);

                for x_ndx in 0..Loot::COLS as i32 {
                    for y_ndx in 0..num_item_rows as i32 {
                        let point = first_item_point
                            + V2::new(x_ndx as f32, y_ndx as f32) * total_slot_size;
                        let is_selected =
                            loot.looking_here && loot.cursor_x == x_ndx && loot.cursor_y == y_ndx;
                        let (bg, outline) = if is_selected {
                            (&light_color, &dark_color)
                        } else {
                            (&dark_color, &light_color)
                        };

                        self.set_fill_color(&bg);
                        self.fill_rect(&AABB {
                            top_left: point,
                            extents: total_slot_size,
                        });

                        if let Some(item) = inventory.item_at_xy(x_ndx, y_ndx) {
                            self.draw_rendering(resources, &point, &item.rendering)?;
                        }

                        self.set_stroke_color(&outline);
                        let item_outline_aabb = AABB {
                            top_left: point + V2::new(slot_padding, slot_padding),
                            extents: slot_size,
                        };
                        self.stroke_rect(&item_outline_aabb);

                        if let Some(item) = inventory.item_at_xy(x_ndx, y_ndx) {
                            if let Some(count) = item.stack {
                                let count_str = format!("x{}", count);
                                let mut count_text = Self::normal_text(&count_str);
                                count_text.size = (12, 12);
                                self.draw_text(
                                    &count_text,
                                    &V2::new(
                                        item_outline_aabb.left() + 1.0,
                                        item_outline_aabb.bottom() - 1.0,
                                    ),
                                )?;
                            }
                        }

                        if is_selected {
                            // Draw the item name at the bottom
                            let empty = "empty";
                            let name = inventory
                                .item_at_xy(x_ndx, y_ndx)
                                .map(|item| item.name.as_str())
                                .unwrap_or(empty);
                            let mut item_name_text = Self::normal_text(name);
                            item_name_text.color = css::light_grey();
                            item_name_text.size = (12, 12);
                            let item_name_text_size = self.measure_text(&item_name_text)?;
                            let item_text_frame_aabb = AABB {
                                top_left: V2::new(inv_bg_aabb.left(), inv_bg_aabb.bottom()),
                                extents: V2::new(
                                    inv_bg_aabb.width(),
                                    item_name_text_size.1 + frame_padding * 2.0,
                                ),
                            };
                            self.set_fill_color(&inv_bg_color);
                            self.fill_rect(&item_text_frame_aabb);
                            let item_name_text_point = V2::new(
                                item_text_frame_aabb.left() + frame_padding,
                                item_text_frame_aabb.top() + frame_padding,
                            );
                            self.draw_text(&item_name_text, &item_name_text_point)?;
                        }
                    }
                }
            } else {
                warn!("looting no inventory");
            }
        }
        Ok(())
    }
}
