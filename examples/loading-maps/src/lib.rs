#[macro_use]
extern crate log;
extern crate console_log;
extern crate console_error_panic_hook;
extern crate mogwai;
extern crate serde;
extern crate serde_json;
extern crate specs;
extern crate old_gods;

use log::Level;
use mogwai::prelude::*;
use old_gods::prelude::*;
use std::{
  panic,
  collections::HashSet,
  sync::{Arc, Mutex}
};
use wasm_bindgen::prelude::*;
use web_sys::{
  HtmlElement,
  HtmlCanvasElement,
  CanvasRenderingContext2d
};

mod ecs;
mod fetch;

use ecs::{ECS, RenderingToggles};


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn maps() -> Vec<String> {
  vec![
    "maps/tiles_test.json".into(),
    "maps/collision_detection.json".into(),
    "maps/door_test.json".into()
  ]
}


#[derive(Clone)]
enum InMsg {
  PostBuild(HtmlElement),
  Load(String),
  LoadError(String),
  Loaded(Tiledmap),
}


#[derive(Clone)]
enum OutMsg {
  Status(String)
}


impl OutMsg {
  fn status_msg(&self) -> Option<String> {
    match self {
      OutMsg::Status(msg) => { Some(msg.clone()) }
      //_ => { None }
    }
  }
}


struct App {
  ecs: Arc<Mutex<ECS<'static, 'static>>>,
  current_map_path: Option<String>
}


impl App {
  fn new(ecs:Arc<Mutex<ECS<'static, 'static>>>) -> App {
    App {
      ecs,
      current_map_path: None
    }
  }
}


impl mogwai::prelude::Component for App {
  type ModelMsg = InMsg;
  type ViewMsg = OutMsg;

  fn update(&mut self, msg: &InMsg, tx_view: &Transmitter<OutMsg>, sub: &Subscriber<InMsg>) {
    match msg {
      InMsg::PostBuild(el) => {
        let canvas:&HtmlCanvasElement =
          el
          .dyn_ref()
          .unwrap_throw();
        let context =
          canvas
          .get_context("2d")
          .unwrap_throw()
          .unwrap_throw()
          .dyn_into::<CanvasRenderingContext2d>()
          .unwrap_throw();
        let mut ecs =
          self
          .ecs
          .try_lock()
          .unwrap_throw();
        ecs.rendering_context = Some(context);
      }
      InMsg::Load(path) => {
        let ecs =
          self
          .ecs
          .try_lock()
          .unwrap_throw();

        self.current_map_path = Some(format!("{}/{}",ecs.base_url, path));
        tx_view.send(&OutMsg::Status(format!("starting load of {}", path)));
        let path = path.clone();
        let base_url = ecs.base_url.clone();
        sub.send_async(async move {
          let tiledmap =
            Tiledmap::from_url(
              &base_url,
              &path,
              |url| {
                fetch::from_url(url)
              }
            ).await;
          match tiledmap {
            Err(msg) => {
              InMsg::LoadError(msg)
            }
            Ok(map) => {
              InMsg::Loaded(map)
            }
          }
        });
      }
      InMsg::LoadError(msg) => {
        self.current_map_path = None;
        tx_view.send(&OutMsg::Status(format!("Loading error:\n{:#?}", msg)));
      }
      InMsg::Loaded(map) => {
        let mut ecs =
          self
          .ecs
          .try_lock()
          .unwrap_throw();
        ecs.world.delete_all();
        let mut loader = MapLoader::new(&mut ecs.world);
        let mut map = map.clone();
        let _ =
          loader
          .insert_map(&mut map, None, None)
          .unwrap_throw();
        let num_entities = {
          let entities =
            ecs
            .world
            .system_data::<Entities>();
          (&entities)
            .join()
            .collect::<Vec<_>>()
            .len()
        };
        tx_view.send(&OutMsg::Status(
          format!(
            "Successfully loaded {} entities from {}",
            num_entities,
            self.current_map_path.as_ref().unwrap(),
          )
        ));
        if ecs.is_debug() {
          let mut ecs_toggles:Write<HashSet<RenderingToggles>> =
            ecs
            .world
            .system_data();
          let map_toggles = RenderingToggles::from_properties(&map.properties);
          *ecs_toggles = map_toggles;
        }
      }
    }
  }

  fn builder(&self, tx: Transmitter<InMsg>, rx: Receiver<OutMsg>) -> GizmoBuilder {
    div()
      .class("container-fluid")
      .with(
        fieldset()
          .with(
            legend()
              .text("Old Gods Map Loader")
          )
          .with_many(
            maps()
              .into_iter()
              .map(|map| {
                div()
                  .with(
                    a()
                      .attribute("href", "#")
                      .text(&map)
                      .tx_on("click", tx.contra_map(move |_| InMsg::Load(map.to_string())))
                  )
              })
              .collect()
          )
          .with(
            pre()
              .rx_text("", rx.branch_filter_map(|msg| msg.status_msg() ))
          )
          .with(
            div()
              .class("embed-responsive embed-responsive-16by9")
              .with(
                canvas()
                  .class("embed-responsive-item")
                  .attribute("id", "screen")
                //.attribute("width", "800")
                //.attribute("height", "600")
                  .tx_post_build(tx.contra_map(|el:&HtmlElement| InMsg::PostBuild(el.clone())))
              )
          )
      )
  }
}


#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
  panic::set_hook(Box::new(console_error_panic_hook::hook));
  console_log::init_with_level(Level::Trace)
    .unwrap();

  let app_ecs = {
    let mut ecs = ECS::new("http://localhost:8888");
    if cfg!(debug_assertions) {
      ecs.set_debug_mode(true);
    }
    Arc::new(Mutex::new(ecs))
  };

  // Set up the game loop
  let ecs = app_ecs.clone();
  request_animation_frame(move || {
    let mut ecs =
      ecs
      .try_lock()
      .unwrap_throw();
    ecs.maintain();
    ecs.render();
    // We always want to reschedule this animation frame
    true
  });

  App::new(app_ecs)
    .into_component()
    .run()
}
