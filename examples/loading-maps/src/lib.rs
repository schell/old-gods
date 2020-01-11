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
//use specs::prelude::*;
use std::{
  panic,
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

use ecs::ECS;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn maps() -> Vec<String> {
  vec![
    "/maps/collision_detection.json".to_string()
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
  rendering_context: Option<CanvasRenderingContext2d>,
  current_map_path: Option<String>
}


impl App {
  fn new(ecs:Arc<Mutex<ECS<'static, 'static>>>) -> App {
    App {
      ecs,
      rendering_context: None,
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
        self.rendering_context = Some(context.clone());
      }
      InMsg::Load(path) => {
        self.current_map_path = Some(path.clone());
        tx_view.send(&OutMsg::Status(format!("starting load of {}", path)));
        let path = path.clone();
        sub.send_async(async move {
          match fetch::from_json(&path).await {
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
        let mut loader = MapLoader::new(&mut ecs.world);
        let mut map = map.clone();
        let _ =
          loader
          .insert_map(&mut map, None, None)
          .unwrap_throw();
        tx_view.send(&OutMsg::Status(format!("Successfully loaded {}", self.current_map_path.as_ref().unwrap())))
      }
    }
  }

  fn builder(&self, tx: Transmitter<InMsg>, rx: Receiver<OutMsg>) -> GizmoBuilder {
    fieldset()
      .with(
        legend()
          .text("Old Gods Map Loader")
      )
      .with_many(
        maps()
          .into_iter()
          .map(|map| {
            trace!("{}", map);
            a()
              .attribute("href", "#")
              .text(&map)
              .tx_on("click", tx.contra_map(move |_| InMsg::Load(map.to_string())))
          })
          .collect()
      )
      .with(
        pre()
          .rx_text("", rx.branch_filter_map(|msg| msg.status_msg() ))
      )
      .with(
        canvas()
          .attribute("id", "screen")
          .attribute("width", "800")
          .attribute("height", "600")
          .tx_post_build(tx.contra_map(|el:&HtmlElement| InMsg::PostBuild(el.clone())))
      )
  }
}


#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
  panic::set_hook(Box::new(console_error_panic_hook::hook));
  console_log::init_with_level(Level::Trace)
    .unwrap();

  let app_ecs = Arc::new(Mutex::new(ECS::new()));

  // Set up the game loop
  let ecs = app_ecs.clone();
  request_animation_frame(move || {
    ecs
      .try_lock()
      .unwrap_throw()
      .maintain();
    true
  });

  App::new(app_ecs)
    .into_component()
    .run()
}
