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
use std::panic;
use wasm_bindgen::prelude::*;
use web_sys::{
  HtmlCanvasElement
};

mod fetch;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


struct WebEngine<'a, 'b> {
  dispatcher: Dispatcher<'a, 'b>,
  world: World
}


impl<'a, 'b> Engine<'a, 'b> for WebEngine<'a, 'b> {
  type Canvas = ();
  type ResourceLoader = ();

  fn new_with(dispatcher_builder: DispatcherBuilder<'a, 'b>) -> WebEngine<'a, 'b> {
    let mut world = World::new();
    let mut dispatcher =
      dispatcher_builder
      //.with_thread_local(SoundSystem::new())
      .with(MapLoadingSystem{ opt_reader: None }, "map", &[])
      .with(ScreenSystem, "screen", &[])
      .with(ActionSystem, "action", &[])
      .with(ScriptSystem, "script", &["action"])
      .with(SpriteSystem, "sprite", &["script"])
      .with(PlayerSystem, "control", &[])
      .with(Physics::new(), "physics", &[])
      .with(AnimationSystem, "animation", &[])
      .with(InventorySystem, "inventory", &[])
      .with(EffectSystem, "effect", &[])
      .with(ItemSystem, "item", &["action", "effect"])
      .with(ZoneSystem, "zone", &[])
      .with(WarpSystem, "warp", &["physics"])
      .with(FenceSystem, "fence", &["physics"])
      .with(TweenSystem, "tween", &[])
      .build();

    dispatcher
      .setup(&mut world);

    // Maintain once so all our resources are created.
    world
      .maintain();

    WebEngine{
      dispatcher,
      world
    }
  }

  fn world(&self) -> &World {
    &self.world
  }

  fn world_mut(&mut self) -> &mut World {
    &mut self.world
  }

  // TODO: Implement canvas 2d rendering
  fn render(&self, world: &mut World) {
    trace!("rendering");
  }
}


fn maps() -> Vec<String> {
  vec![
    "/maps/collision_detection.json".to_string()
  ]
}


#[derive(Clone)]
enum InMsg {
  Startup,
  CreatedCanvas(HtmlCanvasElement),
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
  engine: WebEngine<'static, 'static>,
  canvas: Option<HtmlCanvasElement>,
  current_map_path: Option<String>
}


impl App {
  fn new() -> App {
    App {
      engine: WebEngine::new(),
      canvas: None,
      current_map_path: None
    }
  }
}


impl mogwai::prelude::Component for App {
  type ModelMsg = InMsg;
  type ViewMsg = OutMsg;

  fn update(&mut self, msg: &InMsg, tx_view: &Transmitter<OutMsg>, sub: &Subscriber<InMsg>) {
    match msg {
      InMsg::Startup => {
        let canvas:HtmlCanvasElement = {
          let gizmo =
            canvas()
            .attribute("width", "800")
            .attribute("height", "600")
            .build().unwrap_throw();
          gizmo
            .html_element
            .clone()
            .dyn_into()
            .unwrap()
        };
        sub.send_async(async move { InMsg::CreatedCanvas(canvas) });
      }
      InMsg::CreatedCanvas(c) => {
        self.canvas = Some(c.clone());
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
        let mut loader = MapLoader::new(&mut self.engine.world);
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
  }
}


#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
  panic::set_hook(Box::new(console_error_panic_hook::hook));
  console_log::init_with_level(Level::Trace)
    .unwrap();

  App::new()
    .into_component()
    .run_init(vec![InMsg::Startup])
}
