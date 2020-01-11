use old_gods::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};


trait Resources {
  type Texture;

  fn has_sprite_sheet<S:Into<String>>(&self, s:S) -> bool;
  fn load_sprite_sheet<S:Into<String>>(&mut self, s:S);
  fn take_sprite_sheet<S:Into<String>>(&mut self, s:S) -> Option<Self::Texture>;
  fn put_sprite_sheet<S:Into<String>>(&mut self, s:S, tex:Self::Texture);
}


pub struct HtmlResources {
}

// TODO: Implement Resources for HtmlResources struct
impl Resources for HtmlResources {
  type Texture = HtmlImageElement;

  fn has_sprite_sheet<S:Into<String>>(&self, s:S) -> bool {
    false
  }

  fn load_sprite_sheet<S:Into<String>>(&mut self, s:S) {

  }

  fn take_sprite_sheet<S:Into<String>>(&mut self, s:S) -> Option<Self::Texture> {
    panic!("")
  }

  fn put_sprite_sheet<S:Into<String>>(&mut self, s:S, tex:Self::Texture) {

  }
}

// TODO: Convert draw_sprite to Resources trait stuff
/// Draw a sprite frame at a position.
pub fn draw_sprite(
  context: &CanvasRenderingContext2d,
  src: AABB,
  dest: AABB,
  flip_horizontal: bool,
  flip_vertical: bool,
  flip_diagonal: bool,
  tex: &HtmlImageElement
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

  //canvas
//draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh
  //  .copy_ex(
  //    tex,
  //    src,
  //    dest,
  //    angle,
  //    None,
  //    should_flip_horizontal,
  //    should_flip_vertical
  //  ).unwrap();
}



/// Draw a rendering at a position.
pub fn draw_rendering(
  context: &CanvasRenderingContext2d,
  resources: &mut HtmlResources,
  point: &V2,
  r : &Rendering
) {
  match &r.primitive {
    RenderingPrimitive::TextureFrame(f) => {
      let has_texture =
        resources
        .has_sprite_sheet(&f.sprite_sheet);
      if !has_texture {
        resources
          .load_sprite_sheet(f.sprite_sheet.as_str());
        // Come back later because it's loading etc.
        return;
      }

      let tex =
        resources
        .take_sprite_sheet(&f.sprite_sheet)
        .unwrap_throw();
      let dest =
        AABB::new(
          point.x,
          point.y,
          f.size.0 as f32,
          f.size.1 as f32
        );
      let src =
        AABB::new(
          f.source_aabb.x as f32,
          f.source_aabb.y as f32,
          f.source_aabb.w as f32,
          f.source_aabb.h as f32
        );
      let alpha = context.global_alpha();
      context.set_global_alpha(r.alpha as f64 / 255.0);
      draw_sprite(
        context,
        src,
        dest,
        f.is_flipped_horizontally,
        f.is_flipped_vertically,
        f.is_flipped_diagonally,
        &tex,
      );
      context.set_global_alpha(alpha);
      resources.put_sprite_sheet(&f.sprite_sheet, tex);
    }
    // TODO: Convert RenderingPrimitive::Text(t) to Resources trait stuff
    RenderingPrimitive::Text(t) => {
      //RenderText::texturize_text_if_needed(resources, &t);

      //let texture_key =
      //  t.as_key();
      //let tex =
      //  resources
      //  .texture_manager
      //  .get_cache()
      //  .get(&texture_key)
      //  .expect(&format!(
      //    "Trying to draw some text that has not been texturized:\n{:?}",
      //    texture_key
      //  ));
      //let TextureQuery {width: w, height: h, ..} =
      //  tex
      //  .query();
      //let src =
      //  Rect::new(0, 0, w, h);
      //let dest =
      //  Rect::new(
      //    point.x as i32,
      //    point.y as i32,
      //    w, h
      //  );
      //// draw it
      //draw_sprite(
      //  canvas,
      //  src,
      //  dest,
      //  false, false, false,
      //  &tex
      //);
    }
  }
}



pub fn render(world: &mut World, context: &mut CanvasRenderingContext2d) {

}
