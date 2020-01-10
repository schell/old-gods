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
use web_sys::{FileList, FileReader};
use wasm_bindgen::{
  prelude::*,
  closure::Closure
};


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

  fn render(&self, world: &mut World) {
    trace!("rendering");
  }
}


#[derive(Clone)]
enum InMsg {
  Startup,
  Loaded,
  LoadError,
  Input(HtmlInputElement),
  Chose
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
  file_input: Option<HtmlInputElement>,
  file_reader: FileReader,
  callbacks: Vec<Closure<dyn Fn()>>,
  engine: WebEngine<'static, 'static>
}


impl App {
  fn new() -> App {
    App {
      file_input: None,
      file_reader: FileReader::new().expect("Could not create FileReader"),
      callbacks: vec![],
      engine: WebEngine::new()
    }
  }
}


impl mogwai::prelude::Component for App {
  type ModelMsg = InMsg;
  type ViewMsg = OutMsg;

  fn update(&mut self, msg: &InMsg, tx_view: &Transmitter<OutMsg>, sub: &Subscriber<InMsg>) {
    match msg {
      InMsg::Startup => {
        { // Add the loaded callback
          let sub = sub.clone();
          let cb =
            Closure::wrap(Box::new(move || {
              sub.send_async(async { InMsg::Loaded });
            }) as Box<dyn Fn()>);
          self
            .file_reader
            .add_event_listener_with_callback("load", cb.as_ref().unchecked_ref())
            .unwrap();
          self
            .callbacks
            .push(cb);
        }
        { // Add the err'd callback
          let sub = sub.clone();
          let cb =
            Closure::wrap(Box::new(move || {
              sub.send_async(async { InMsg::LoadError });
            }) as Box<dyn Fn()>);
          self
            .file_reader
            .add_event_listener_with_callback("error", cb.as_ref().unchecked_ref())
            .unwrap();
          self
            .callbacks
            .push(cb);
        }
      }
      InMsg::Loaded => {
        tx_view.send(&OutMsg::Status("loaded!".into()));
        trace!("{}", self.file_reader.result().unwrap_throw().as_string().unwrap_throw());
        let _world = self.engine.world();
      }
      InMsg::LoadError => {
        let err =
          self
          .file_reader
          .error();
        tx_view.send(&OutMsg::Status(format!("Loading error:\n{:#?}", err)));
      }
      InMsg::Input(input) => {
        self.file_input = Some(input.clone());
      }

      InMsg::Chose => {
        self
          .file_input
          .iter()
          .for_each(|input: &HtmlInputElement| {
            let files:Option<FileList> =
              input.files();
            let file =
              files
              .map(|files| files.get(0))
              .unwrap_or(None);
            if let Some(file) = file {
              // Load the map file
              self
                .file_reader
                .read_as_text(&file.into())
                .expect("Could not load");
            }
          })
      }
    }
  }

  fn builder(&self, tx: Transmitter<InMsg>, rx: Receiver<OutMsg>) -> GizmoBuilder {
    fieldset()
      .with(
        legend()
          .text("Old Gods Map Loader")
      )
      .with(
        pre()
          .rx_text("", rx.branch_filter_map(|msg| msg.status_msg() ))
      )
      .with(
        div()
          .with(
            label()
              .attribute("for", "map_file")
              .text("Tiled.json map:")
          )
          .with(
            input()
              .attribute("name", "file_select")
              .attribute("type", "file")
              .attribute("cursor", "pointer")
              .id("map_file")
              .tx_post_build(
                tx.contra_map(|el:&HtmlElement| {
                  InMsg::Input(el.dyn_ref::<HtmlInputElement>().unwrap().clone())
                })
              )
              .tx_on("change", tx.contra_map(|_| InMsg::Chose ))
          )
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
