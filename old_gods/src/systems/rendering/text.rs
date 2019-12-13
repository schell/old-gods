//use sdl2::render::*;
//use sdl2::rect::Rect;
use std::path::Path;

use super::super::super::geom::V2;
use super::super::super::resource_manager::*;
use super::record::*;
use super::render;


pub struct RenderText;


impl<'ctx, 'res> RenderText {

  /// Texturize some text
  pub fn texturize_text(
    resources: &'res mut Sdl2Resources<'ctx>,
    texture_key: &String,
    t: &Text
  ) {
    // The path to the font is actually inside the
    // font directory
    let fonts_dir =
      resources
      .font_directory
      .clone();
    let mut descriptor =
      t.font.clone();
    descriptor.path =
      Path::new(&fonts_dir)
      .join(Path::new(&descriptor.path))
      .with_extension("ttf")
      .to_str()
      .unwrap()
      .to_string();
    // Load the font
    let font =
      resources
      .font_manager
      .load(&descriptor)
      .unwrap();
    // Generate the texture
    let surface =
      font
      .render(&t.text.as_str())
      .blended(t.color)
      .map_err(|e| e.to_string())
      .unwrap();
    let mut texture =
      resources
      .texture_creator
      .create_texture_from_surface(&surface)
      .map_err(|e| e.to_string())
      .unwrap();
    texture
      .set_blend_mode(BlendMode::Blend);
    texture
      .set_alpha_mod(t.color.a);
    // Give the texture to the texture manager
    resources
      .texture_manager
      .put_resource(&texture_key, texture);
  }


  /// Texturize some text if needed
  pub fn texturize_text_if_needed(
    resources: &'res mut Sdl2Resources<'ctx>,
    text: &Text
  ) -> &'res Texture<'ctx> {
    // Maybe we've already drawn this text before, generate
    // the key it would/will live under
    let texture_key =
      format!("{:?}", text);
    // Determine if we've already generated a texture for this
    // text rendering
    let has_text =
      resources
      .texture_manager
      .get_cache()
      .contains_key(&texture_key);
    if !has_text {
      Self::texturize_text(resources, &texture_key, text);
    }
    resources
      .texture_manager
      .get_cache()
      .get(&texture_key)
      .expect("Impossible")
  }


  /// Get the drawn size of some text
  pub fn text_size(
    resources: &'res mut Sdl2Resources<'ctx>,
    text: &Text
  ) -> (u32, u32) {
    let tex =
      Self::texturize_text_if_needed(resources, text);
    let TextureQuery{ width, height, ..} =
      tex
      .query();
    (width, height)
  }


  /// Draw some text, returns the destination Rect where the
  /// text was drawn.
  pub fn draw_text(
    canvas: &mut WindowCanvas,
    resources: &'res mut Sdl2Resources<'ctx>,
    pos: &V2,
    text: &Text
  ) -> Rect {
    let tex =
      Self::texturize_text_if_needed(resources, text);
    let TextureQuery{ width, height, ..} =
      tex
      .query();
    let dest =
      Rect::new(
        pos.x as i32,
        pos.y as i32,
        width, height
      );
    render::draw_sprite(
      canvas,
      Rect::new(0, 0, width, height),
      dest,
      false, false, false,
      &tex
    );
    dest
  }

}
