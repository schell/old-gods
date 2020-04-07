/// Contains functions for rendering the portions of the user interface that
/// appear on the map.
//use sdl2::render::*;
//use sdl2::pixels::Color;
//use sdl2::rect::Rect;
use specs::prelude::*;

use super::super::super::components::*;
use super::super::super::geom::*;
use super::super::super::resource_manager::{FontDetails, Sdl2Resources};
use super::super::super::utils::CanBeEmpty;
use super::super::physics::*;
use super::super::screen::Screen;
use super::render::*;
use super::record::*;
use super::text::RenderText;


pub struct RenderUI;


impl<'ctx, 'res, 'sys> RenderUI {

  pub fn fancy_font() -> FontDetails {
    // TODO: Allow the UI font to be customized
    FontDetails {
      path: "GothicPixels".to_string(),
      size: 16
    }
  }


  pub fn fancy_text(msg: &str) -> Text {
    Text {
      text: msg.to_string(),
      font: Self::fancy_font(),
      color: Color::rgb(255, 255, 255),
      size: (16, 16)
    }
  }


  pub fn normal_font() -> FontDetails {
    // TODO: Allow the UI font to be customized
    FontDetails {
      path: "sans-serif".to_string(),
      size: 16
    }
  }


  pub fn normal_text(msg: &str) -> Text {
    Text {
      text: msg.to_string(),
      font: Self::normal_font(),
      color: Color::rgb(255, 255, 255),
      size: (16, 16)
    }
  }


  pub fn make_loot_rendering (
    loot: &Loot,
    inventories: &ReadStorage<'sys, Inventory>,
    items: &ReadStorage<'sys, Item>,
    renderings: &ReadStorage<'sys, Rendering>,
    names: &ReadStorage<'sys, Name>,
  ) -> LootRendering {
    let mk_items =
      |inventory: &Inventory| -> Vec<InventoryItem> {
        let mut inv_items = vec![];
        for ent in &inventory.items {
          let Name(name) =
            names
            .get(*ent)
            .expect("An item is missing a Name.")
            .clone();
          let item =
            items
            .get(*ent)
            .expect("An item does not have an Item component");
          let usable =
            item
            .usable;
          let count =
            item
            .stack
            .unwrap_or(1);
          let frame =
            renderings
            .get(*ent)
            .expect("An item is missing its Rendering component.")
            .as_frame()
            .expect("An item's Rendering is not a TextureFrame")
            .clone();
          inv_items.push(
            InventoryItem {
              name,
              frame,
              usable,
              count
            }
          );
        }
        inv_items
      };
    let mk_inv =
      |ent: Entity| {
        let Name(name) =
          names
          .get(ent)
          .expect("Cannot draw a loot without a Name")
          .clone();
        InventoryRendering {
          items:
          mk_items(
            inventories
              .get(ent)
              .expect("Cannot draw a loot without an Inventory")
          ),
          name
        }
      };
    let inventory_a =
      mk_inv(loot.looter);
    let inventory_b =
      loot
      .inventory
      .map(mk_inv);
    LootRendering {
      inventory_a,
      inventory_b,
      cursor_in_a: loot.is_looking_in_own_inventory,
      index: loot.index.clone()
    }
  }


  /// Draw loots involving a player that are on the screen
  pub fn draw_loots (
    canvas: &mut WindowCanvas,
    resources: &'res mut Sdl2Resources,
    exiles: &ReadStorage<'sys, Exile>,
    inventories: &ReadStorage<'sys, Inventory>,
    items: &ReadStorage<'sys, Item>,
    loots: &ReadStorage<'sys, Loot>,
    names: &ReadStorage<'sys, Name>,
    positions: &ReadStorage<'sys, Position>,
    renderings: &ReadStorage<'sys, Rendering>,
    _screen: &Screen,
    players: &ReadStorage<'sys, Player>
  ) {
    for (loot, _) in (loots, !exiles).join() {
      let has_position =
        positions.contains(loot.looter)
        || (loot.inventory.is_some() && positions.contains(loot.inventory.unwrap()));
      let has_player =
        players.contains(loot.looter)
        || (loot.inventory.is_some() && players.contains(loot.inventory.unwrap()));
      if !has_position || !has_player {
        continue;
      }
      let mut players_vec =
        vec![
          players
            .get(loot.looter)
            .cloned(),
        ];
        loot
        .inventory
        .map(|i| {
          let player =
            players
            .get(i)
            .cloned();
          players_vec
            .push(player);
        });
      let players_vec:Vec<Player> =
        players_vec
        .into_iter()
        .filter_map(|t| t)
        .collect();
      let may_player:Option<&Player> =
        players_vec
        .first();
      if may_player.is_some() {
        let loot_rendering =
          Self::make_loot_rendering(
            &loot,
            inventories,
            items,
            renderings,
            names
          );
        RenderInventory::draw_loot(
          canvas,
          resources,
          &V2::new(10.0, 10.0),
          loot_rendering,
        );
      }
    }
  }

  pub fn action_button_color(btn: &ActionButton) -> Color {
    match btn {
      ActionButton::A => {
        Color::rgb(50, 229, 56)
      },
      ActionButton::B => {
        Color::rgb(202, 16, 16)
      }
      ActionButton::X => {
        Color::rgb(16, 124, 202)
      }
      ActionButton::Y => {
        Color::rgb(197, 164, 23)
      }
    }
  }


  pub fn action_button_text(btn: &ActionButton) -> String {
    match btn {
      ActionButton::A => {
        "A"
      },
      ActionButton::B => {
        "B"
      }
      ActionButton::X => {
        "X"
      }
      ActionButton::Y => {
        "Y"
      }
    }.to_string()
  }

  /// Draw an action button at a point with an optional message to the right.
  pub fn draw_action_button(
    canvas: &mut WindowCanvas,
    resources: &'res mut Sdl2Resources<'ctx>,
    btn: ActionButton,
    point: &V2,
    msg: &Option<String>
  ) -> Rect {
    let mut btn_text =
      Self::fancy_text(&Self::action_button_text(&btn).as_str());
    btn_text.color = Self::action_button_color(&btn);
    let dest =
      RenderText::draw_text(
        canvas,
        resources,
        point,
        &btn_text
      );
    let btn_rect = dest;
    let text_rect =
      if let Some(text) = msg {
        let text =
          Self::normal_text(&text.as_str());
        RenderText::draw_text(
          canvas,
          resources,
          &V2::new(dest.right() as f32, point.y),
          &text
        )
      } else {
        btn_rect
      };
    btn_rect.union(text_rect)
  }


  /// Draw an Action.
  pub fn draw_action(
    canvas: &mut WindowCanvas,
    resources: &'res mut Sdl2Resources<'ctx>,
    point: &V2,
    action: &Action
  ) {
    let msg:Option<String> =
      action
      .text
      .non_empty()
      .map(|s| s.clone());
    Self::draw_action_button(
      canvas,
      resources,
      ActionButton::A,
      &(*point - V2::new(7.0, 7.0)),
      &msg
    );
  }


  /// Draw the UI elements.
  pub fn draw_ui (
    canvas: &mut WindowCanvas,
    resources: &mut Sdl2Resources<'ctx>,
    screen: &Screen,
    actions: &ReadStorage<'sys, Action>,
    entities: &Entities<'sys>,
    exiles: &ReadStorage<'sys, Exile>,
    inventories: &ReadStorage<'sys, Inventory>,
    items: &ReadStorage<'sys, Item>,
    loots: &ReadStorage<'sys, Loot>,
    names: &ReadStorage<'sys, Name>,
    offsets: &ReadStorage<'sys, OriginOffset>,
    positions: &ReadStorage<'sys, Position>,
    renderings: &ReadStorage<'sys, Rendering>,
    players: &ReadStorage<'sys, Player>,
  ) {
    for (ent, position, action) in (entities, positions, actions).join() {
      if !action.display_ui {
        continue;
      }

      let offset =
        offsets
        .get(ent)
        .map(|o| o.clone())
        .unwrap_or(OriginOffset(V2::new(0.0, 0.0)));
      let pos =
        position.0 + offset.0;
      let pos =
        screen
        .from_map(&pos);
      Self::draw_action(canvas, resources, &pos, &action);
    }

    Self::draw_loots(
      canvas,
      resources,
      exiles,
      inventories,
      items,
      loots,
      names,
      positions,
      renderings,
      screen,
      players
    );
  }

}
