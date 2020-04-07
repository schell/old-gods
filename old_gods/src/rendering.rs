use super::prelude::{
    Color, V2, AABB,
};

/// TODO: Abstract rendering into Renderer trait and implementations.
/// TODO: Get the CSS colors module from gelatin and port it here.
pub trait RenderingContext {
    type Image;
    type Font;

    fn set_fill_color(&mut self, color: &Color);
    fn fill_rect(&mut self, aabb: &AABB);

    fn set_font(&mut self, font: &Self::Font);
    fn fill_text(&mut self, text: &str, point: &str);

    fn measure_text(&mut self, name: &str, path: &str, size: u32, text: &str);

    fn set_stroke_color(&mut self, color: &Color);
    fn stroke_lines(&mut self, lines: &Vec<V2>);
    fn stroke_rect(&mut self, aabb: &AABB);

    fn draw_image(&mut self, img: &Self::Image, src: &AABB, destination: &AABB);

    // These functions from loading-maps/src/ecs/render.rs can be written in terms of
    // the previous:
    //
    // fn draw_text(&mut self, text: &Text, pos: &V2);
    // fn draw_rendering<Rsrc:Resources<Image>>(&mut self, &mut img_rsrc:Rsrc, point: &V2, rendering: &Rendering);
}
