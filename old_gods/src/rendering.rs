use super::prelude::{
    Color, FontDetails, Rendering, RenderingPrimitive, Resources, Text, AABB, V2,
};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

pub mod standard;


/// TODO: Get the CSS colors module from gelatin and port it here.
pub trait RenderingContext {
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
    fn draw_context(
        &mut self,
        context: &Self,
        destination: &AABB
    ) -> Result<(), String>;

    fn font_details_to_font(&mut self, font_details: &FontDetails) -> Self::Font;

    // These remaining methods are provided by default, but may be overridden
    // by instances

    fn draw_text(&mut self, text: &Text, pos: &V2) -> Result<(), String> {
        let font = self.font_details_to_font(&text.font);
        self.set_font(&font);
        self.fill_text(text.text.as_str(), pos)
    }

    fn draw_sprite(
        &mut self,
        src: AABB,
        destination: AABB,
        flip_horizontal: bool,
        flip_vertical: bool,
        flip_diagonal: bool,
        tex: &Self::Image,
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
        self.draw_image(tex, &src, &destination)
    }

    fn draw_rendering<T>(
        &mut self,
        rsc: &mut T,
        point: &V2,
        rendering: &Rendering,
    ) -> Result<(), String>
    where
        T: Resources<Self::Image>,
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
                    let alpha = self.global_alpha();
                    self.set_global_alpha(rendering.alpha as f64 / 255.0);
                    self.draw_sprite(
                        src,
                        dest,
                        f.is_flipped_horizontally,
                        f.is_flipped_vertically,
                        f.is_flipped_diagonally,
                        &tex,
                    )?;
                    self.set_global_alpha(alpha);
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

    fn measure_text(&mut self, text: &Text) -> Result<(f32, f32), String> {
        let font = self.font_details_to_font(&text.font);
        self.size_of_text(&font, &text.text)
    }
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

    fn set_context_size(self: &mut CanvasRenderingContext2d, (w, h):(u32, u32)) -> Result<(), String> {
        let canvas = self
            .canvas()
            .ok_or("rendering context has no canvas".to_string())?;
        canvas.set_width(w);
        canvas.set_height(h);
        Ok(())
    }

    fn clear(&mut self) -> Result<(), String> {
        self.set_fill_style(&JsValue::from(Color::rgba(0, 0, 0, 0)));
        let (w, h) = self.context_size()?;
        self.fill_rect(&AABB::new(
            0.0, 0.0,
            w as f32, h as f32
        ));
        Ok(())
    }

    fn set_fill_color(self: &mut CanvasRenderingContext2d, color: &Color) {
        //self.set_global_alpha(color.a as f64 / 255.0);
        self.set_fill_style(&JsValue::from(color.clone()));
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

    fn set_font(&mut self, font: &Self::Font) {
        CanvasRenderingContext2d::set_font(self, &format!("{}px {}", font.size, font.path));
    }

    fn fill_text(&mut self, text: &str, point: &V2) -> Result<(), String> {
        CanvasRenderingContext2d::fill_text(self, text, point.x as f64, point.y as f64)
            .map_err(|e| format!("cannot fill text: {:#?}", e))
    }

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
        CanvasRenderingContext2d::set_stroke_style(self, &color.clone().into());
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
            &context.canvas().ok_or("can't draw map to window".to_string())?,
            dest.top_left.x as f64,
            dest.top_left.y as f64,
            dest.width() as f64,
            dest.height() as f64,
        ).map_err(|e| format!("can't draw context: {:#?}", e))
    }

    fn font_details_to_font(&mut self, font_details: &FontDetails) -> Self::Font {
        font_details.clone()
    }
}
