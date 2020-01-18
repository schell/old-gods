use old_gods::prelude::*;
use std::{
  collections::HashMap,
  sync::{
    Arc,
    Mutex
  },
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{
  JsCast,
  UnwrapThrowExt
};
use web_sys::{
  window,
  CanvasRenderingContext2d,
  ErrorEvent,
  EventTarget,
  HtmlImageElement,
  Window,
};


#[derive(Clone)]
pub enum LoadStatus {
  None,
  Started ,
  Complete,
  Error(String)
}


trait Resources {
  type Texture;

  fn status_sprite_sheet<S:Into<String>>(&self, s:S) -> LoadStatus;
  fn load_sprite_sheet<S:Into<String>>(&mut self, s:S);
  fn take_sprite_sheet(&mut self, s:&str) -> Option<Self::Texture>;
  fn put_sprite_sheet<S:Into<String>>(&mut self, s:S, tex:Self::Texture);
}


pub struct Callbacks(Arc<Closure<dyn Fn(JsValue)>>, Arc<Closure<dyn Fn(JsValue)>>);


pub struct HtmlResources {
  sprite_sheets: HashMap<String, Arc<Mutex<(LoadStatus, Option<HtmlImageElement>)>>>,
  callbacks: HashMap<String, Callbacks>
}


impl HtmlResources {
  pub fn new() -> Self {
    HtmlResources {
      sprite_sheets: HashMap::new(),
      callbacks: HashMap::new()
    }
  }
}


// TODO: Test Resources for HtmlResources implementation
// Possibly by testing draw_sprite
impl Resources for HtmlResources {
  type Texture = HtmlImageElement;

  fn status_sprite_sheet<S:Into<String>>(&self, s:S) -> LoadStatus {
    self
      .sprite_sheets
      .get(&s.into())
      .map(|payload| {
        let status_and_may_img =
          payload
          .try_lock()
          .unwrap();
        status_and_may_img.0.clone()
      })
      .unwrap_or(LoadStatus::None)
  }

  fn load_sprite_sheet<S:Into<String>>(&mut self, s:S) {
    let path = s.into();
    let img =
      window().unwrap_throw()
      .document().unwrap_throw()
      .create_element("img").unwrap_throw()
      .dyn_into::<HtmlImageElement>().unwrap_throw();
    img.set_src(&path);
    let status =
      Arc::new(Mutex::new((
        LoadStatus::Started,
        img.clone(),
      )));
    let target:&EventTarget =
      img
      .dyn_ref()
      .unwrap_throw();
    let load_status = status.clone();
    let load =
      Closure::wrap(Box::new(move |_:JsValue| {
        let mut status_and_img =
          load_status
          .try_lock()
          .unwrap_throw();
        status_and_img.0 = LoadStatus::Complete;
      }) as Box<dyn Fn(JsValue)>);
    let err_status = status.clone();
    let err =
      Closure::wrap(Box::new(move |event:JsValue| {
        let mut status_and_img =
          err_status
          .try_lock()
          .unwrap_throw();
        let event =
          event
          .dyn_into::<ErrorEvent>()
          .unwrap_throw();
        let msg =
          event
          .error()
          .as_string()
          .unwrap_or("unknown error".into());
        status_and_img.0 = LoadStatus::Error(msg);
      }) as Box<dyn Fn(JsValue)>);
    target
      .add_event_listener_with_callback("load", load.as_ref().unchecked_ref())
      .unwrap();
    target
      .add_event_listener_with_callback("error", err.as_ref().unchecked_ref())
      .unwrap();
    self
      .callbacks
      .insert(path, Callbacks (Arc::new(load), Arc::new(err)));
  }

  fn take_sprite_sheet(&mut self, s:&str) -> Option<Self::Texture> {
    let _ = self.callbacks.remove(s)?;
    let status_and_img = self.sprite_sheets.remove(s)?;
    let status_and_img = status_and_img.try_lock().ok()?;
    status_and_img.1.clone()
  }

  fn put_sprite_sheet<S:Into<String>>(&mut self, s:S, tex:Self::Texture) {
    let status_and_img =
      Arc::new(Mutex::new((LoadStatus::Complete, Some(tex))));
    self
      .sprite_sheets
      .insert(s.into(), status_and_img);
  }
}


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

  context
    .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
      tex,
      src.top_left.x as f64,
      src.top_left.y as f64,
      src.width() as f64,
      src.height() as f64,
      dest.top_left.x as f64,
      dest.top_left.y as f64,
      dest.width() as f64,
      dest.height() as f64,
    )
    .unwrap_throw();
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
      let texture_status =
        resources
        .status_sprite_sheet(&f.sprite_sheet);
      match texture_status {
        LoadStatus::None => {
          // Load it and come back later
          resources
            .load_sprite_sheet(f.sprite_sheet.as_str());
        }
        LoadStatus::Started => {
          // Come back later because it's loading etc.
          return;
        }
        LoadStatus::Complete => {}
        LoadStatus::Error(msg) => {
          warn!("sprite sheet loading error: {}", msg);
        }
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
    RenderingPrimitive::Text(t) => {
      let alpha = context.global_alpha();
      context.set_global_alpha(t.color.a as f64 / 255.0);
      context.set_fill_style(
        &JsValue::from_str(&format!("rgb({}, {}, {})", t.color.r, t.color.g, t.color.b))
      );
      context.set_font(&format!("{}px {}", t.font.size, t.font.path));
      context
        .fill_text(&t.text, point.x as f64, point.y as f64)
        .unwrap_throw();
      context.set_global_alpha(alpha);
    }
  }
}


// TODO: Implement rendering each Rendering from the world
pub fn render(world: &mut World, context: &mut CanvasRenderingContext2d) {

}
