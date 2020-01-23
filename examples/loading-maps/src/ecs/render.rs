use old_gods::prelude::*;
use std::{
  collections::{
    HashMap,
    HashSet
  },
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


pub fn set_fill_color(color: &Color, context: &CanvasRenderingContext2d) {
  context.set_fill_style(
    &JsValue::from_str(&format!("rgb({}, {}, {})", color.r, color.g, color.b))
  );
}


pub fn set_stroke_color(color: &Color, context: &CanvasRenderingContext2d) {
  context.set_stroke_style(
    &JsValue::from_str(&format!("rgb({}, {}, {})", color.r, color.g, color.b))
  );
}


// TODO: Rendering functions should return Result<_, JsValue>
// Rendering may produce an error. Let's track that.
pub fn draw_text(t: &Text, point: &V2, context: &CanvasRenderingContext2d) {
  let point =
    V2 {
      x: point.x,
      // CanvasRenderingContext2d draws text with the origin at the baseline
      y: point.y + t.font.size as f32
    };
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


pub fn measure_text(t: &Text, context: &CanvasRenderingContext2d) -> (f32, f32) {
  context.set_font(&format!("{}px {}", t.font.size, t.font.path));
  let num_lines =
    t
    .text
    .lines()
    .count();
  let height =
    t.font.size * num_lines as u16;
  let metrics =
    context
    .measure_text(&t.text)
    .unwrap_throw();
  let width = metrics.width();
  (width as f32, height as f32)
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
      draw_text(t, point, context);
    }
  }
}

// TODO: Debug rendering
type RenderData<'s> = (
  Read<'s, BackgroundColor>,
  Write<'s, Screen>,
  //Read<'s, UI>,
  Entities<'s>,
  ReadStorage<'s, Position>,
  ReadStorage<'s, OriginOffset>,
  ReadStorage<'s, Rendering>,
  ReadStorage<'s, ZLevel>,
  ReadStorage<'s, Exile>,
  ReadStorage<'s, Shape>,
);


pub fn render(world: &mut World, resources: &mut HtmlResources, context: &mut CanvasRenderingContext2d) {
  let
    ( background_color,
      mut screen,
      //_ui,
      entities,
      positions,
      offset_store,
      renderings,
      zlevels,
      exiles,
      shapes,
    ):RenderData = world.system_data();

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
//       .set_fill_color(Color::rgb(0, 0, 0));
//     canvas
//       .clear();
//     canvas
//       .copy(&target, Some(src), Some(dest))
//       .unwrap();

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
//       &velocities,
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
}


#[derive(Debug, Clone,  Hash, PartialEq, Eq)]
/// Various toggles to display or hide things during rendering.
/// Toggling the rendering of various debug infos can be done by adding a custom
/// property to your map file.
pub enum RenderingToggles {
  /// Toggle marking actions.
  Actions,

  /// Toggle rendering positions.
  Positions,

  /// Toggle rendering barriers.
  Barriers,

  /// Toggle rendering the AABBTree.
  AABBTree,

  /// Toggle rendering velocities.
  Velocities,

  /// Toggle rendering zlevels.
  ZLevels,

  /// Toggle marking players.
  Players,

  /// Toggle marking the screen
  Screen,

  /// Toggle displaying the FPS.
  FPS,

  /// Render zones
  Zones,

  /// Fences
  Fences,

  /// Display the apparent entity count
  EntityCount,

  /// Display collision system information
  CollisionInfo,

  /// Display all shapes
  Shapes
}


impl RenderingToggles {
  pub fn property_map() -> HashMap<String, RenderingToggles> {
    use RenderingToggles::*;
    let props = vec![
      Actions,
      Positions,
      Barriers,
      AABBTree,
      Velocities,
      ZLevels,
      Players,
      Screen,
      FPS,
      Zones,
      Fences,
      EntityCount,
      CollisionInfo,
      Shapes,
    ];
    props
      .into_iter()
      .map(|t| (t.property_str().to_string(), t))
      .collect()
  }


  pub fn property_str(&self) -> &str {
    use RenderingToggles::*;
    match self {
      Actions =>       { "toggle_rendering_actions" }
      Positions =>     { "toggle_rendering_positions" }
      Barriers =>      { "toggle_rendering_barriers" }
      AABBTree =>      { "toggle_rendering_aabb_tree" }
      Velocities =>    { "toggle_rendering_velocities" }
      ZLevels =>       { "toggle_rendering_z_levels" }
      Players =>       { "toggle_rendering_players" }
      Screen =>        { "toggle_rendering_screen" }
      FPS =>           { "toggle_rendering_fps" }
      Zones =>         { "toggle_rendering_zones" }
      Fences =>        { "toggle_rendering_fences" }
      EntityCount =>   { "toggle_rendering_entity_count" }
      CollisionInfo => { "toggle_rendering_collision_info" }
      Shapes =>        { "toggle_rendering_shapes" }
    }
  }

  pub fn from_properties(props:&Vec<Property>) -> HashSet<RenderingToggles> {
    let toggles = Self::property_map();
    let mut set = HashSet::new();
    for prop in props {
      toggles
        .get(&prop.name)
        .into_iter()
        .for_each(|toggle:&RenderingToggles| {
          prop
            .value
            .as_bool()
            .into_iter()
            .for_each(|should_set| {
              if should_set {
                set.insert(toggle.clone());
              }
            });
        })
    }
    set
  }
}


pub type DebugRenderingData<'s> =
  ( Read<'s, AABBTree>,
    Entities<'s>,
    Read<'s, HashSet<RenderingToggles>>,
    Read<'s, FPSCounter>,
    Read<'s, Screen>,
    ReadStorage<'s, Velocity>,
    ReadStorage<'s, Barrier>,
    ReadStorage<'s, Exile>,
    ReadStorage<'s, Item>,
    ReadStorage<'s, Tags>,
    ReadStorage<'s, Player>,
    ReadStorage<'s, Position>,
    ReadStorage<'s, OriginOffset>,
    ReadStorage<'s, Action>,
    ReadStorage<'s, Name>,
    ReadStorage<'s, Looting>,
    ReadStorage<'s, Inventory>,
    ReadStorage<'s, Zone>,
    ReadStorage<'s, Fence>,
    ReadStorage<'s, Shape>,
    ReadStorage<'s, StepFence>,
    ReadStorage<'s, ZLevel>
  );


fn debug_font_details() -> FontDetails {
  FontDetails {
    path: "sans-serif".to_string(),
    size: 16
  }
}

fn debug_text(text: &str) -> Text {
  Text {
    text: text.to_string(),
    font: debug_font_details(),
    color: Color::rgb(255, 255, 255),
    size: (16, 16)
  }
}

/// Construct a vector of lines that form an arrow from p1 to p2
pub fn arrow_lines(p1: V2, p2: V2) -> Vec<V2> {
  let zero = V2::new(0.0, 0.0);
  let n  =
    (p2 - p1)
    .normal()
    .unitize()
    .unwrap_or(zero);
  let p3 = p2 - (p2 - p1).unitize().unwrap_or(zero).scalar_mul(5.0);
  let p4 = p3 + n.scalar_mul(5.0);
  let p5 = p3 - n.scalar_mul(5.0);
  vec![p1, p2, p4, p5, p2]
}

/// Construct a vector of lines that form a kind of hour glass shape.
pub fn point_lines(p: V2) -> Vec<V2> {
  let tl =
    p + V2::new(-10.0, -10.0);
  let tr =
    p + V2::new(10.0, -10.0);
  let bl =
    p + V2::new(-10., 10.0);
  let br =
    p + V2::new(10.0, 10.0);
  vec![
    tl.clone(), tr,
    bl, br, tl
  ]
}


pub fn draw_lines(lines: &Vec<V2>, context: &CanvasRenderingContext2d) {
  let mut iter = lines.iter();
  iter
    .next()
    .iter()
    .for_each(|point| context.move_to(point.x as f64, point.y as f64));
  iter
    .for_each(|point| context.line_to(point.x as f64, point.y as f64));

  context.stroke();
}

fn draw_map_aabb(aabb: &AABB, screen: &Screen, context: &CanvasRenderingContext2d) {
  let dbg_aabb =
    AABB::from_points(
      screen.map_to_window(&aabb.lower()),
      screen.map_to_window(&aabb.upper())
    );
  context.stroke_rect(
    dbg_aabb.top_left.x as f64,
    dbg_aabb.top_left.y as f64,
    dbg_aabb.extents.x as f64,
    dbg_aabb.extents.y as f64,
  );
}


fn draw_map_arrow(from: V2, to: V2, screen: &Screen, context: &CanvasRenderingContext2d) {
  let lines =
    arrow_lines(
      screen
        .map_to_window(&from),
      screen
        .map_to_window(&to)
    );
  draw_lines(&lines, context);
}


fn draw_map_point(at: V2, screen: &Screen, context: &CanvasRenderingContext2d) {
  let lines =
    point_lines(
      screen
        .map_to_window(&at),
    );
  draw_lines(&lines, context);
}



pub fn render_debug(
  world: &mut World,
  _resources: &mut HtmlResources,
  context: &mut CanvasRenderingContext2d
) {
  let
    ( aabb_tree,
      entities,
      toggles,
      fps,
      screen,
      velocities,
      barriers,
      exiles,
      _items,
      _tag_store,
      players,
      positions,
      offsets,
      actions,
      names,
      _loots,
      _inventories,
      zones,
      fences,
      shapes,
      step_fences,
      zlevels
    ):DebugRenderingData = world.system_data();
    // Get player 0's z
    let player =
      (&players, &zlevels)
      .join()
      .filter(|(p, _)| p.0 == 0)
      .collect::<Vec<_>>()
      .first()
      .cloned();

    let next_rect =
      if toggles.contains(&RenderingToggles::FPS) {
        let fps = fps.current_fps();
        let fps_text =
          debug_text(format!("FPS:{:.2}", fps).as_str());
        let pos =
          V2::new(0.0, 0.0);
        draw_text(&fps_text, &pos, context);
        let size = measure_text(&fps_text, context);
        AABB {
          top_left: pos,
          extents: V2 {
            x: size.0,
            y: size.1
          }
        }
      } else {
        AABB::identity()
      };

    if toggles.contains(&RenderingToggles::EntityCount) {
      let count:u32 =
        (&entities)
        .join()
        .fold(
          0,
          |n, _| n + 1
        );
      let text =
        debug_text(format!("Entities: {}", count).as_str());
      let pos =
        V2::new(0.0, next_rect.bottom() as f32);
      draw_text(&text, &pos, context);
    }

    if toggles.contains(&RenderingToggles::Velocities) {
      let joints = (&entities, &positions, &velocities).join();
      for (entity, position, velo) in joints {
        let v  = if velo.0.magnitude() < 1e-10 {
          return;
        } else {
          velo.0
        };
        let offset:V2 =
          entity_local_origin(entity, &shapes, &offsets);
        let p1 =
          screen
          .map_to_window(&(position.0 + offset));
        let p2 = p1 + v;
        let lines = arrow_lines(p1, p2);
        set_stroke_color(&Color::rgb(255, 255, 0), context);
        draw_lines(&lines, context);
      }
    }

    if toggles.contains(&RenderingToggles::AABBTree) {
      let mbrs =
        aabb_tree
        .rtree
        .lookup_in_rectangle(&screen.aabb().to_mbr());
      for EntityBounds{ bounds: mbr, entity_id: id } in mbrs {
        let entity = entities.entity(*id);
        let z =
          zlevels
          .get(entity)
          .or(
            player.map(|p| p.1)
          )
          .cloned()
          .unwrap_or(ZLevel(0.0));
        let alpha =
          if player.is_some() {
            if z.0 == (player.unwrap().1).0 {
              255
            } else {
              50
            }
          } else {
            255
          };
        let color =
          if exiles.contains(entity) {
            Color::rgba(255, 0, 255, alpha)
          } else {
            Color::rgba(255, 255, 0, alpha)
          };
        let aabb =
          AABB::from_mbr(&mbr);
        let aabb =
          AABB::from_points(
            screen
              .map_to_window(&aabb.top_left),
            screen
              .map_to_window(&aabb.lower())
          );

        set_stroke_color(&color, context);
        context.stroke_rect(
          aabb.top_left.x as f64, aabb.top_left.y as f64,
          aabb.extents.x as f64, aabb.extents.y as f64,
        );
        if let Some(name) = names.get(entity) {
          let p = V2::new(aabb.top_left.x, aabb.bottom());
          let mut text = debug_text(name.0.as_str());
          text.color = color;
          draw_text(&text, &p, context);
        }

      }
    }

    if toggles.contains(&RenderingToggles::Zones) {
      let joints = (&entities, &positions, &zones, &shapes).join();
      for (entity, Position(p), _, shape) in joints {
        let mut color =
          Color::rgb(139, 175, 214);
        let alpha =
          if exiles.contains(entity) {
            128
          } else {
            255
          };
        color.a = alpha;
        let extents =
          shape
          .extents();
        let aabb = AABB::from_points(
          screen.map_to_window(p),
          screen.map_to_window(&(*p + extents))
        );
        set_fill_color(&color, context);
        context.fill_rect(
          aabb.top_left.x as f64,
          aabb.top_left.y as f64,
          aabb.extents.x as f64,
          aabb.extents.y as f64
        );
        if let Some(name) = names.get(entity) {
          let p = V2::new(aabb.top_left.x, aabb.bottom());
          let mut text = debug_text(name.0.as_str());
          text.color = color;
          draw_text(&text, &p, context);
        }
      }
    }

    if toggles.contains(&RenderingToggles::Fences) {
      let aabb =
        screen
        .aabb();
      let filter_fence =
        |p: &Position, points: &Vec<V2> | -> bool {
          for point in points {
            if aabb.contains_point(&(p.0 + *point)) {
              return true
            }
          }
          false
        };
      let mut fences:Vec<(Entity, &Position, &Fence, Color)> =
        (&entities, &positions, &fences)
        .join()
        .filter(|(_,p,f)| {
          filter_fence(p, &f.points)
        })
        .map(|(e,p,f)| (e, p, f, Color::rgb(153, 102, 255)))
        .collect();
      let mut step_fences:Vec<(Entity, &Position, &Fence, Color)> =
        (&entities, &positions, &step_fences)
        .join()
        .filter(|(_, p, f)| {
          filter_fence(p, &f.0.points)
        })
        .map(|(e,p,f)| (e,p,&f.0,Color::rgb(102, 0, 255)))
        .collect();
      fences
        .append(&mut step_fences);

      for (entity, &Position(pos), fence, color) in fences {
        let pos =
          screen
          .map_to_window(&pos);
        let lines:Vec<V2> =
          fence
          .points
          .iter()
          .map(|p| pos + *p)
          .collect();
        set_fill_color(&color, context);
        draw_lines(&lines, context);
        if let Some(name) = names.get(entity) {
          let text = debug_text(name.0.as_str());
          draw_text(&text, &pos, context);
        }
      }
    }

    //if self.toggles.contains(&RenderingToggles::Positions) {
    //  self.canvas.set_fill_color(Color::rgb(0, 0, 255));
    //  let p = position.0 + *offset;
    //  self.canvas.draw_rect(
    //    Rect::new(
    //      (p.x - focus_offset.x) as i32 - 2,
    //      (p.y - focus_offset.y) as i32 - 2,
    //      4,
    //      4
    //    )
    //  ).expect("Could not draw position.");

    //  let pos_str = format!(
    //    "({}, {}, z{})",
    //    position.0.x.round() as i32,
    //    position.0.y.round() as i32,
    //    may_z.unwrap_or(&ZLevel(0.0)).0
    //  );
    //  self.draw_text(&pos_str, position.0);
    //} else if self.toggles.contains(&RenderingToggles::ZLevels) {
    //  let z = may_z.unwrap_or(&ZLevel(0.0)).0;
    //  self.draw_text(&format!("z{}", z), position.0 - *focus_offset);
    //}

    if toggles.contains(&RenderingToggles::Players)
      && !toggles.contains(&RenderingToggles::Barriers) {
        let joints = (&entities, &positions, &players).join();
        for (entity, position, _player) in joints {
          let offset = offsets
            .get(entity)
            .map(|o| o.0)
            .unwrap_or(V2::origin());
          let p =
            screen
            .map_to_screen(&(position.0 + offset));
          set_fill_color(&Color::rgb(0, 255, 255), context);
          context.fill_rect(
              (p.x - 24.0) as f64,
              (p.y - 24.0) as f64,
              48.0,
              48.0
          );
          //let text =
          //  Self::debug_text(format!("{:?}", player));
          //RenderText::draw_text(canvas, resources, &p);
        }
      }

    if toggles.contains(&RenderingToggles::Screen) {
      let screen_aabb = screen.aabb();
      let window_aabb =
        AABB::from_points(
          screen
            .map_to_window(&screen_aabb.lower()),
          screen
            .map_to_window(&screen_aabb.upper())
        );
      set_stroke_color(&Color::rgb(0, 255, 0), context);
      context.stroke_rect(
        window_aabb.top_left.x as f64,
        window_aabb.top_left.y as f64,
        window_aabb.extents.x as f64,
        window_aabb.extents.y as f64
      );

      let focus_aabb =
        screen
        .focus_aabb();
      let window_focus_aabb =
        AABB::from_points(
          screen
            .map_to_window(&focus_aabb.top_left),
          screen
            .map_to_window(&focus_aabb.lower())
        );
      context.stroke_rect(
        window_focus_aabb.top_left.x as f64,
        window_focus_aabb.top_left.y as f64,
        window_focus_aabb.extents.x as f64,
        window_focus_aabb.extents.y as f64
      );
    }

    if toggles.contains(&RenderingToggles::Actions) {
      for (ent, _, Position(pos)) in (&entities, &actions, &positions).join() {
        let is_exiled =
          exiles
          .contains(ent);

        let color = if is_exiled {
          Color::rgb(255, 255, 255)
        } else {
          Color::rgb(252, 240, 5)
        };

        let a =
          screen.map_to_screen(pos);
        let b =
          a + V2::new(10.0, -20.0);
        let c =
          a + V2::new(-10.0, -20.0);
        set_fill_color(&color, context);
        let lines = vec![a, b, c, a];
        draw_lines(&lines, context);
      }
    }

    if toggles.contains(&RenderingToggles::Shapes) {
      for (shape, Position(p)) in (&shapes, &positions).join() {
        let color = Color::rgb(128, 128, 255);
        set_fill_color(&color, context);

        let lines:Vec<V2> =
          shape
          .vertices_closed()
          .into_iter()
          .map(|v| screen.map_to_window(&(*p + v)))
          .collect();
        draw_lines(&lines, context);
      }
    }

    let show_collision_info = toggles.contains(&RenderingToggles::CollisionInfo);
    if toggles.contains(&RenderingToggles::Barriers) || show_collision_info {
      let player_z:f32 =
        (&players, &zlevels)
        .join()
        .filter(|(p, _)| p
                .0 == 0 )
        .collect::<Vec<_>>()
        .first()
        .clone()
        .map(|(_, z)| z.0)
        .unwrap_or(0.0);

      let joints = (&entities, &barriers, &shapes, &positions, &zlevels).join();
      for (ent, Barrier, shape, Position(p), z) in joints {
        let is_exiled = exiles
          .get(ent)
          .map(|_| true)
          .unwrap_or(false);
        let alpha =
          if z.0 == player_z {
            255
          } else {
            50
          };
        let color =
          if is_exiled {
            Color::rgba(255, 255, 255, alpha)
          } else {
            Color::rgba(255, 0, 0, alpha)
          };
        set_stroke_color(&color, context);

        let lines:Vec<V2> =
          shape
          .vertices_closed()
          .into_iter()
          .map(|v| screen.map_to_window(&(*p + v)))
          .collect();
        draw_lines(&lines, context);

        if show_collision_info {
          // Draw the potential separating axes
          let axes =
            shape
            .potential_separating_axes();
          let midpoints =
            shape
            .midpoints();
          // light red
          let color =
            Color::rgb(255, 128, 128);
          set_stroke_color(&color, context);
          for (axis, midpoint) in axes.into_iter().zip(midpoints) {
            let midpoint =
              screen
              .screen_to_window(&midpoint);
            let lines = arrow_lines(midpoint, midpoint + (axis.scalar_mul(20.0)));
            draw_lines(&lines, context);
          }

          // Draw its collision with other shapes
          let aabb =
            shape
            .aabb()
            .translate(&p);
          let broad_phase_collisions:Vec<(Entity, AABB)> =
            aabb_tree
            .query(&entities, &aabb, &ent);
          broad_phase_collisions
            .into_iter()
            .for_each(|(other_ent, other_aabb)| {
              // Draw the union of their aabbs to show the
              // broad phase collision
              let color =
                Color::rgb(255, 128, 64); // orange
              set_stroke_color(&color, context);
              let union = AABB::union(&aabb, &other_aabb);
              draw_map_aabb(&union, &screen, context);

              // Find out if they actually collide and what the
              // mtv is
              let other_shape =
                &shapes
                .get(other_ent)
                .expect("Can't get other shape");
              let other_position =
                positions
                .get(other_ent);
              if other_position.is_none() {
                // This is probably an item that's in an inventory.
                return;
              }
              let other_position =
                other_position
                .unwrap();
              let mtv =
                shape
                .mtv_apart(*p, &other_shape, other_position.0);
              mtv
                .map(|mtv| {
                  set_stroke_color(&Color::rgb(255, 255, 255), context);
                  draw_map_point(
                    other_aabb.center(),
                    &screen,
                    context
                  );
                  draw_map_arrow(
                    other_aabb.center(),
                    other_aabb.center() + mtv,
                    &screen,
                    context
                  );
                });
            });
        }
      }
    }
}


pub trait Renderer {
  type Context;
  type Resources;

  fn set_fill_color(color:&Color, context:&Self::Context);
  fn set_stroke_color(color:&Color, context:&Self::Context);
  fn stroke_lines(lines:&Vec<V2>, context:&Self::Context);
  fn stroke_rect(aabb:&AABB, context:&Self::Context);
  fn fill_rect(aabb:&AABB, context:&Self::Context);
  fn draw_rendering(r:&Rendering, point:&V2, context:&Self::Context, resources: &mut Self::Resources);
}
