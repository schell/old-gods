use old_gods::{
    prelude::{Exile, Resources, Join, Player, Position, ReadStorage, World, V2, Name, DefaultRenderingContext},
    rendering::*,
};
use std::ops::{Deref, DerefMut};
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

mod inventory;
use super::systems::inventory::{Inventory, Loot};

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
        WebRenderingContext(DefaultRenderingContext{context})
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
        R: Resources<<Self::Ctx as RenderingContext>::Image>
    {
        self.deref_mut().render_ui(world, resources, viewport_to_context)?;

        let (exiles, inventories, loots, names, positions, players): (
            ReadStorage<Exile>,
            ReadStorage<Inventory>,
            ReadStorage<Loot>,
            ReadStorage<Name>,
            ReadStorage<Position>,
            ReadStorage<Player>,
        ) = world.system_data();

        // Draw lootings involving a player that are on the screen
        for (loot, _) in (&loots, !&exiles).join() {
            let has_position = positions.contains(loot.looter)
                || (loot.inventory.is_some() && positions.contains(loot.inventory.unwrap()));
            let has_player = players.contains(loot.looter)
                || (loot.inventory.is_some() && players.contains(loot.inventory.unwrap()));
            if !has_position || !has_player {
                continue;
            }
            let mut players_vec = vec![players.get(loot.looter).cloned()];
            loot.inventory.map(|i| {
                let player = players.get(i).cloned();
                players_vec.push(player);
            });
            let players_vec: Vec<Player> = players_vec.into_iter().filter_map(|t| t).collect();
            let may_player: Option<&Player> = players_vec.first();
            if may_player.is_some() {
                let loot_rendering = inventory::make_loot_rendering(&loot, &inventories, &names);
                inventory::draw_loot(self, resources, &V2::new(10.0, 10.0), loot_rendering)?;
            }
        }
        Ok(())
    }
}
