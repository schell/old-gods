use old_gods::prelude::*;
use std::{
  collections::HashMap,
  cmp::Ordering,
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
    trace!("loading sprite sheet: {}", &path);
    let img =
      window().unwrap_throw()
      .document().unwrap_throw()
      .create_element("img").unwrap_throw()
      .dyn_into::<HtmlImageElement>().unwrap_throw();
    img.set_src(&path);
    let status =
      Arc::new(Mutex::new((
        LoadStatus::Started,
        Some(img.clone()),
      )));
    let target:&EventTarget =
      img
      .dyn_ref()
      .unwrap_throw();
    let load_status = status.clone();
    let load_path = path.clone();
    let load =
      Closure::wrap(Box::new(move |_:JsValue| {
        let mut status_and_img =
          load_status
          .try_lock()
          .expect("Could not acquire lock - load_sprite_sheet::load");
        trace!("  loading {} complete", &load_path);
        status_and_img.0 = LoadStatus::Complete;
      }) as Box<dyn Fn(JsValue)>);
    let err_status = status.clone();
    let err_path = path.clone();
    let err =
      Closure::wrap(Box::new(move |event:JsValue| {
        let mut status_and_img =
          err_status
          .try_lock()
          .expect("Could not acquire lock - load_sprite_sheet::err");
        trace!("error event: {:#?}", event);
        let event =
          event
          .dyn_into::<web_sys::Event>()
          .expect("Error is not an Event");
        let msg = format!("failed loading {}: {}", &err_path, event.type_());
        trace!("  loading {} erred: {}", &err_path, &msg);
        status_and_img.0 = LoadStatus::Error(msg);
        status_and_img.1 = None;
      }) as Box<dyn Fn(JsValue)>);
    target
      .add_event_listener_with_callback("load", load.as_ref().unchecked_ref())
      .unwrap();
    target
      .add_event_listener_with_callback("error", err.as_ref().unchecked_ref())
      .unwrap();
    self
      .callbacks
      .insert(path.clone(), Callbacks (Arc::new(load), Arc::new(err)));
    self
      .sprite_sheets
      .insert(path, status);
  }

  fn take_sprite_sheet(&mut self, s:&str) -> Option<Self::Texture> {
    let _ = self.callbacks.remove(s);
    let status_and_img = self.sprite_sheets.remove(s)?;
    let status_and_img = status_and_img.try_lock().ok()?;
    status_and_img.1.clone()
  }

  fn put_sprite_sheet<S:Into<String>>(&mut self, s:S, tex:Self::Texture) {
    let path = s.into();
    let status_and_img =
      Arc::new(Mutex::new((LoadStatus::Complete, Some(tex))));
    self
      .sprite_sheets
      .insert(path, status_and_img);
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
          return;
        }
        LoadStatus::Started => {
          // Come back later because it's loading etc.
          return;
        }
        LoadStatus::Complete => {}
        LoadStatus::Error(msg) => {
          warn!("sprite sheet loading error: {}", msg);
          return;
        }
      }

      let tex =
        resources
        .take_sprite_sheet(&f.sprite_sheet)
        .expect("Could not take sprite sheet");
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

// TODO: Debug rendering
type RenderData<'s> = (
  //Read<'s, AABBTree>,
  Read<'s, BackgroundColor>,
  //Read<'s, HashSet<RenderingToggles>>,
  Write<'s, Screen>,
  //Read<'s, UI>,
  //Read<'s, FPSCounter>,
  Entities<'s>,
  ReadStorage<'s, Position>,
  //ReadStorage<'s, Velocity>,
  ReadStorage<'s, OriginOffset>,
  ReadStorage<'s, Rendering>,
  //ReadStorage<'s, Barrier>,
  ReadStorage<'s, ZLevel>,
  ReadStorage<'s, Exile>,
  //ReadStorage<'s, Item>,
  //ReadStorage<'s, Tags>,
  //ReadStorage<'s, Player>,
  ReadStorage<'s, Shape>,
  //ReadStorage<'s, Action>,
  //ReadStorage<'s, Name>,
  //ReadStorage<'s, Looting>,
  //ReadStorage<'s, Inventory>,
  //ReadStorage<'s, Item>,
  //ReadStorage<'s, Zone>,
  //ReadStorage<'s, Fence>,
  //ReadStorage<'s, StepFence>,
);


pub fn render(world: &mut World, resources: &mut HtmlResources, context: &mut CanvasRenderingContext2d) {
  let ( //aabb_tree,
        background_color,
        //toggles,
        mut screen,
        //_ui,
        //fps,
        entities,
        positions,
        //velo_store,
        offset_store,
        renderings,
        //barriers,
        zlevels,
        exiles,
        //items,
        //_tag_store,
        //toons,
        shapes,
        //actions,
        //names,
        //loots,
        //inventories,
        //_items,
        //zones,
        //fences,
        //step_fences
      ): RenderData = world.system_data();
  // Set the screen's size and the window size, return the screen's map aabb
  let screen_aabb = {
    let canvas =
      context
      .canvas()
      .unwrap_throw();
    screen.window_size = (canvas.width(), canvas.height());
    screen.aabb()
  };

  // Get all the on screen things to render.
  // Order the things by bottom to top, back to front.
  let mut ents:Vec<_> =
    (&entities, &positions, &renderings, !&exiles)
    .join()
    .filter(|(_, p, r, _)| {
      // Make sure we can see this thing (that its destination aabb intersects
      // the screen)
      let (w, h) =
        r.size();
      let aabb =
        AABB {
          top_left: p.0,
          extents: V2::new(w as f32, h as f32)
        };
      screen_aabb.collides_with(&aabb)
        || aabb.collides_with(&screen_aabb)
    })
    .map(|(ent, p, r, _)| {
      let offset:V2 =
        entity_local_origin(ent, &shapes, &offset_store);
      let pos =
        screen
        .map_to_screen(&p.0);
      (ent, Position(pos), offset, r, zlevels.get(ent))
    })
    .collect();
  ents
    .sort_by( |(_, p1, offset1, _, mz1), (_, p2, offset2, _, mz2)| {
      let lvl = ZLevel(0.0);
      let z1 = mz1.unwrap_or(&lvl);
      let z2 = mz2.unwrap_or(&lvl);
      if z1.0 < z2.0 {
        Ordering::Less
      } else if z1.0 > z2.0 {
        Ordering::Greater
      } else if p1.0.y + offset1.y < p2.0.y + offset2.y {
        Ordering::Less
      } else if p1.0.y + offset1.y > p2.0.y + offset2.y {
        Ordering::Greater
      } else {
        Ordering::Equal
      }
    });

  // Render into our render target texture
  context
    .set_global_alpha(background_color.0.a as f64 / 255.0);
  context
    .set_fill_style(
      &JsValue::from_str(
        &format!(
          "rgb({}, {}, {})",
          background_color.0.r,
          background_color.0.g,
          background_color.0.b,
        )
      )
    );
  context
    .fill_rect(
      0.0, 0.0,
      screen_aabb.width() as f64, screen_aabb.height() as f64
    );

  // Draw map renderings
  ents
    .iter()
    .for_each(|(_entity, p, _, r, _)| {
      draw_rendering(context, resources, &p.0, r);
    });
//
//     // Now use our render target to draw inside the screen, upsampling to the
//     // screen size
//     let src =
//       AABB::new(
//         0.0, 0.0,
//         self.resolution.0 as f32, self.resolution.1 as f32
//       );
//     let dest =
//       AABB::from_points(
//         screen.screen_to_window(&src.top_left),
//         screen.screen_to_window(&src.extents)
//       )
//       .to_rect();
//     let src =
//       src
//       .to_rect();
//     canvas
//       .set_draw_color(Color::rgb(0, 0, 0));
//     canvas
//       .clear();
//     canvas
//       .copy(&target, Some(src), Some(dest))
//       .unwrap();
//
//     RenderDebug::draw_debug(
//       &mut canvas,
//       &mut resources,
//       &toggles,
//       &aabb_tree,
//       &actions,
//       &fps,
//       &screen,
//       &entities,
//       &names,
//       &positions,
//       &offset_store,
//       &velo_store,
//       &toons,
//       &barriers,
//       &shapes,
//       &exiles,
//       &zones,
//       &fences,
//       &step_fences,
//       &zlevels
//     );
//
//     RenderUI::draw_ui(
//       &mut canvas,
//       &mut resources,
//       &screen,
//       &actions,
//       &entities,
//       &exiles,
//       &inventories,
//       &items,
//       &loots,
//       &names,
//       &offset_store,
//       &positions,
//       &renderings,
//       &toons
//     );
//
//     canvas
//       .present();
//
//     // Give all the things back to ourself
//     self.canvas = Some(canvas);
//     self.target = Some(target);
//     self.resources = Some(resources);
//   }
}
