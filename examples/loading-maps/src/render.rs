use old_gods::{
    prelude::{
        DefaultRenderingContext, Exile, Join, Name, Player, Position, Read, ReadStorage, Resources,
        World, V2,
    },
    rendering::*,
};
use std::ops::{Deref, DerefMut};
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

use super::components::inventory::Inventory;
use super::systems::looting::Loot;

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

        let (exiles, inventories, loots, names, positions, players): (
            ReadStorage<Exile>,
            ReadStorage<Inventory>,
            Read<Vec<Loot>>,
            ReadStorage<Name>,
            ReadStorage<Position>,
            ReadStorage<Player>,
        ) = world.system_data();

        // Draw lootings involving a player that are on the screen
        for loot in loots.iter() {
            let has_position = positions.contains(loot.ent_of_inventory_here);
            let player:Option<&Player> = players
                .get(loot.ent_of_inventory_here)
                .or_else(|| loot.ent_of_inventory_there.map(|ent| players.get(ent)).flatten());
            if !has_position || player.is_none() {
                continue;
            }
            if let Some(player) = player {
                let loot_rendering = inventory::make_loot_rendering(&loot, &inventories, &names);
                inventory::draw_loot(self, resources, &V2::new(10.0, 10.0), loot_rendering)?;
            }
        }
        Ok(())
    }
}
