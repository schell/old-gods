//! Everything needed to render anything in the Old Gods engine.
//!
//! The functions in Render are rendering functions that don't require
//! interacting with the ECS or loading textures - just the rendering primitives.
//use sdl2::render::*;
//use sdl2::rect::Rect;
//use sdl2::render::Texture;

use super::{
  super::super::{
    color::Color,
    geom::*,
  },
  record::*,
  text::RenderText,
  ui::RenderUI,
};
  //use super::super::super::resource_manager::Sdl2Resources;


// TODO: Render static tiles together into a texture.
// This will cut down on calls to canvas.copy and bump up the FPS.
// The only way I can think to do this is by using a special layer property that
// tells the engine that it's okay to batch tiles together.


// /// Draw a rendering at a position.
// pub fn draw_rendering(
//   canvas: &mut WindowCanvas,
//   resources: &mut Sdl2Resources,
//   point: &V2,
//   r : &Rendering
// ) {
//   match &r.primitive {
//     RenderingPrimitive::TextureFrame(f) => {
//       let has_texture =
//         resources
//         .texture_manager
//         .get_cache()
//         .contains_key(&f.sprite_sheet);
//       if !has_texture {
//         resources
//           .texture_manager
//           .load(f.sprite_sheet.as_str())
//           .unwrap();
//       }
//
//       let mut tex =
//         resources
//         .texture_manager
//         .take_resource(&f.sprite_sheet)
//         .unwrap();
//       let dest =
//         AABB::new(
//           point.x as i32,
//           point.y as i32,
//           f.size.0,
//           f.size.1
//         );
//       let src =
//         AABB::new(
//           f.source_aabb.x as i32,
//           f.source_aabb.y as i32,
//           f.source_aabb.w,
//           f.source_aabb.h
//         );
//       let alpha =
//         tex
//         .alpha_mod();
//       tex
//         .set_alpha_mod(r.alpha);
//       draw_sprite(
//         canvas,
//         src,
//         dest,
//         f.is_flipped_horizontally,
//         f.is_flipped_vertically,
//         f.is_flipped_diagonally,
//         &tex,
//       );
//       tex
//         .set_alpha_mod(alpha);
//       resources
//         .texture_manager
//         .put_resource(f.sprite_sheet.as_str(), tex);
//     }
//     RenderingPrimitive::Text(t) => {
//       RenderText::texturize_text_if_needed(resources, &t);
//
//       let texture_key =
//         t.as_key();
//       let tex =
//         resources
//         .texture_manager
//         .get_cache()
//         .get(&texture_key)
//         .expect(&format!(
//           "Trying to draw some text that has not been texturized:\n{:?}",
//           texture_key
//         ));
//       let TextureQuery {width: w, height: h, ..} =
//         tex
//         .query();
//       let src =
//         Rect::new(0, 0, w, h);
//       let dest =
//         Rect::new(
//           point.x as i32,
//           point.y as i32,
//           w, h
//         );
//       // draw it
//       draw_sprite(
//         canvas,
//         src,
//         dest,
//         false, false, false,
//         &tex
//       );
//     }
//   }
// }


/// A renderable inventory item.
pub struct InventoryItem {
  pub name:  String,
  pub frame: TextureFrame,
  pub usable: bool,
  pub count: usize
}


/// A renderable inventory.
pub struct InventoryRendering {
  pub items: Vec<InventoryItem>,
  pub name: String
}


/// A renderable looting operation.
pub struct LootRendering {
  pub inventory_a: InventoryRendering,
  pub inventory_b: Option<InventoryRendering>,
  pub cursor_in_a: bool,
  pub index: Option<usize>
}


impl LootRendering {
  /// Return the item currently under the cursor
  pub fn current_item(&self) -> Option<&InventoryItem> {
    self
      .index
      .map(|ndx| {
        let inv =
          if self.inventory_b.is_none()
               || self.cursor_in_a {
            &self
              .inventory_a
          } else {
            &self
              .inventory_b
              .as_ref()
              .unwrap()
          };
        inv
          .items
          .get(ndx)
      })
      .unwrap_or(None)
  }
}


pub struct RenderInventory;


impl<'ctx, 'res> RenderInventory {

  /// Draw a player inventory
  pub fn draw_looting (
    canvas: &mut WindowCanvas,
    resources: &'res mut Sdl2Resources<'ctx>,
    point: &V2,
    loot: LootRendering,
  ) {
    let width = 150.0;
    let item_height = 50;
    let name_height = 20;
    let mut invs =
      vec![
        (true, &loot.inventory_a, *point),
      ];
    if loot.inventory_b.is_some() {
      invs.push((false, &loot.inventory_b.as_ref().unwrap(), (*point + V2::new(150.0, 0.0))));
    }
    for (is_a, inv, origin) in invs {
      // Draw the background
      canvas.set_draw_color(Color::rgba(0, 0, 0, 128));
      let bg_height =
        if inv.items.len() > 0 {
          inv.items.len() as u32 * item_height
        } else {
          item_height
        } + name_height;
      let bg_rect =
        Rect::new(
          origin.x as i32,
          origin.y as i32,
          width as u32,
          bg_height
        );
      canvas
        .fill_rect(bg_rect)
        .expect("Could not draw inventory background.");
      canvas
        .set_draw_color(Color::rgba(255, 255, 225, 255));
      canvas
        .draw_rect(bg_rect)
        .expect("Could not draw inventory background.");

      // Draw each item
      for (item, n) in inv.items.iter().zip(0..inv.items.len()) {
        let pos = origin + V2::new(0.0, name_height as f32 + item_height as f32 * n as f32);
        let tex =
          resources
          .texture_manager
          .load(&item.frame.sprite_sheet)
          .unwrap();
        let src =
          Rect::new(
            item.frame.source_aabb.x as i32,
            item.frame.source_aabb.y as i32,
            item.frame.source_aabb.w,
            item.frame.source_aabb.h,
          );
        let dest =
          Rect::new(
            pos.x as i32,
            pos.y as i32,
            item.frame.size.0,
            item.frame.size.1
          );
        draw_sprite(
          canvas,
          src,
          dest,
          item.frame.is_flipped_horizontally,
          item.frame.is_flipped_vertically,
          item.frame.is_flipped_diagonally,
          tex
        );
        let text_pos = pos + V2::new(48.0, 10.0);
        let name =
          item
          .name
          .clone();
        let text =
          RenderUI::fancy_text(name.as_str());
        let item_aabb =
          RenderText::draw_text(canvas, resources, &text_pos, &text);
        if item.count > 1 {
          let pos =
            V2::new(item_aabb.left() as f32, item_aabb.bottom() as f32 + 2.0);
          let mut text =
            RenderUI::normal_text(&format!("x{}", item.count));
          text.font.size = 12;
          RenderText::draw_text(canvas, resources, &pos, &text);
        }
      }

      // Draw the inventory name
      let inv_name_text =
        RenderUI::fancy_text(inv.name.as_str());
      RenderText::draw_text(canvas, resources, &(origin + V2::new(2.0, 2.0)), &inv_name_text);

      // Draw the cursor
      let looking_at_this_inv =
        loot.cursor_in_a == is_a;
      if looking_at_this_inv && inv.items.len() > 0 && loot.index.is_some() {
        let ndx =
          loot
          .index
          .expect("Impossible");
        canvas.set_draw_color(Color::rgb(0, 255, 0));
        let cursor_y =
          name_height as i32 + origin.y as i32 + ndx as i32 * 50;
        canvas.draw_rect(
          Rect::new(origin.x as i32 + 1, cursor_y, 149, 50)
        ).expect("Could not draw inventory cursor.");

      } else if inv.items.len() == 0 {
        // Draw the empty inventory
        let mut text =
          RenderUI::fancy_text("(empty)");
        text.color = Color::rgb(128, 128, 128);
        RenderText::draw_text(
          canvas,
          resources,
          &(origin + V2::new(45.0, 32.0)),
          &text
        );
      }
    }
    // Draw the close inventory msg
    let a_btn_rect = {
      let msg =
        Some("close".to_string());
      let items_len =
        usize::max(
          loot
            .inventory_a
            .items
            .len(),
          loot
            .inventory_b
            .as_ref()
            .map(|i| i.items.len())
            .unwrap_or(0)
        );
      let items_len =
        usize::max(1, items_len);
      let msg_y =
        item_height as f32 * items_len as f32 + name_height as f32;
      let msg_point =
        *point + V2::new(4.0, msg_y);
      RenderUI::draw_action_button(
        canvas,
        resources,
        ActionButton::Y,
        &msg_point,
        &msg
      )
    };

    // Draw the "use" item inventory msg
    let current_item_is_usable =
      loot
      .current_item()
      .map(|item| item.usable)
      .unwrap_or(false);

    if current_item_is_usable {
      let msg =
        Some("use".to_string());
      let pos =
        V2::new(a_btn_rect.right() as f32, a_btn_rect.top() as f32);
      RenderUI::draw_action_button(
        canvas,
        resources,
        ActionButton::X,
        &pos,
        &msg
      );
    }

  }
}


/// Draw a black border frame around the whole screen.
pub fn draw_frame(canvas: &mut WindowCanvas, ww: u32, wh: u32) {
  canvas
    .set_draw_color(Color::rgb(0, 0, 0));
  canvas
    .fill_rect(Rect::new(0, 0, ww, 48))
    .expect("Could not draw frame.");
  canvas.fill_rect(Rect::new(0, wh as i32 - 48, ww, 48))
    .expect("Could not draw frame.");
  canvas.fill_rect(Rect::new(0, 0, 48, wh))
    .expect("Could not draw frame.");
  canvas.fill_rect(Rect::new(ww as i32 - 48, 0, 48, wh))
    .expect("Could not draw frame.");
}

/// Draw a sprite frame at a position.
pub fn draw_sprite(
  canvas: &mut WindowCanvas,
  src: Rect,
  dest: Rect,
  flip_horizontal: bool,
  flip_vertical: bool,
  flip_diagonal: bool,
  tex: &Texture
) {
  let src = Some(src);
  let dest = Some(dest);

  let mut should_flip_horizontal = false;
  let should_flip_vertical;
  let mut angle = 0.0;

  match (flip_diagonal, flip_horizontal, flip_vertical) {
    (true, true, true) => {
      angle = -90.0;
      should_flip_vertical = true;
    },
    (true, a, b) => {
      angle = -90.0;
      should_flip_vertical = !b;
      should_flip_horizontal = a;
    }
    (false, a, b) => {
      should_flip_horizontal = a;
      should_flip_vertical = b;
    }
  }

  canvas
    .copy_ex(
      tex,
      src,
      dest,
      angle,
      None,
      should_flip_horizontal,
      should_flip_vertical
    ).unwrap();
}
