use log::{trace, Level};
use mogwai::prelude::*;
use old_gods::prelude::*;
use std::{
    collections::HashSet,
    panic,
    sync::{Arc, Mutex},
};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement};

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
        "maps/tiles_test.json".into(),
        "maps/collision_detection.json".into(),
        "maps/full_test.json".into(),
    ]
}


#[derive(Clone)]
enum InMsg {
    PostBuild(HtmlCanvasElement),
    Load(String),
    LoadError(String),
    Loaded(Tiledmap),
}


#[derive(Clone)]
enum OutMsg {
    Status(String),
}


impl OutMsg {
    fn status_msg(&self) -> Option<String> {
        match self {
            OutMsg::Status(msg) => Some(msg.clone()), //_ => { None }
        }
    }
}


struct App {
    ecs: Arc<Mutex<ECS<'static, 'static>>>,
    current_map_path: Option<String>,
}


impl App {
    fn new(ecs: Arc<Mutex<ECS<'static, 'static>>>) -> App {
        App {
            ecs,
            current_map_path: None,
        }
    }
}


impl mogwai::prelude::Component for App {
    type ModelMsg = InMsg;
    type ViewMsg = OutMsg;
    type DomNode = HtmlElement;

    fn update(&mut self, msg: &InMsg, tx_view: &Transmitter<OutMsg>, sub: &Subscriber<InMsg>) {
        match msg {
            InMsg::PostBuild(canvas) => {
                let context = canvas
                    .get_context("2d")
                    .expect("can't call get_context('2d')")
                    .expect("can't get rendering context")
                    .dyn_into::<CanvasRenderingContext2d>()
                    .expect("can't coerce rendering context");
                context.set_image_smoothing_enabled(false);
                let mut ecs = self.ecs.try_lock().expect("no lock on ecs");
                ecs.rendering_context = Some(context);
                ecs.set_resolution(canvas.width(), canvas.height());
            }
            InMsg::Load(path) => {
                let ecs = self.ecs.try_lock().expect("no lock on ecs");

                self.current_map_path = Some(format!("{}/{}", ecs.base_url, path));
                tx_view.send(&OutMsg::Status(format!("starting load of {}", path)));
                let path = path.clone();
                let base_url = ecs.base_url.clone();
                sub.send_async(async move {
                    let tiledmap = Tiledmap::from_url(&base_url, &path, fetch::from_url).await;
                    match tiledmap {
                        Err(msg) => InMsg::LoadError(msg),
                        Ok(map) => InMsg::Loaded(map),
                    }
                });
            }
            InMsg::LoadError(msg) => {
                self.current_map_path = None;
                tx_view.send(&OutMsg::Status(format!("Loading error:\n{:#?}", msg)));
            }
            InMsg::Loaded(map) => {
                let mut ecs = self.ecs.try_lock().expect("no lock on ecs");
                ecs.world.delete_all();

                if let Some((width, height)) = map.get_suggested_viewport_size() {
                    trace!("got map viewport size: {} {}", width, height);
                    ecs.set_resolution(width, height);
                }
                let num_entities = {
                    let entities = ecs.world.system_data::<Entities>();
                    (&entities).join().collect::<Vec<_>>().len()
                };
                tx_view.send(&OutMsg::Status(format!(
                    "Successfully loaded {} entities from {}",
                    num_entities,
                    self.current_map_path.as_ref().unwrap(),
                )));
                if ecs.is_debug() {
                    let mut ecs_toggles: Write<HashSet<RenderingToggles>> = ecs.world.system_data();
                    let map_toggles = RenderingToggles::from_properties(&map.properties);
                    *ecs_toggles = map_toggles;
                }
                {
                    let mut data: ecs::systems::tiled::InsertMapData = ecs.world.system_data();
                    ecs::systems::tiled::insert_map(map, &mut data);
                }

                ecs.restart_time();
            }
        }
    }

    fn view(&self, tx: Transmitter<InMsg>, rx: Receiver<OutMsg>) -> Gizmo<HtmlElement> {
        div().class("container-fluid").with(
            maps()
                .into_iter()
                .fold(
                    fieldset().with(legend().text("Old Gods Map Loader")),
                    |fieldset, map| {
                        fieldset.with(div().with(a().attribute("href", "#").text(&map).tx_on(
                            "click",
                            tx.contra_map(move |_| InMsg::Load(map.to_string())),
                        )))
                    },
                )
                .with(pre().rx_text("", rx.branch_filter_map(|msg| msg.status_msg())))
                .with(
                    div().class("embed-responsive embed-responsive-16by9").with(
                        canvas()
                            .downcast::<HtmlCanvasElement>()
                            .ok()
                            .expect("not a canvas")
                            .class("embed-responsive-item")
                            .attribute("id", "screen")
                            .attribute("width", "1600")
                            .attribute("height", "900")
                            .tx_post_build(tx.contra_map(|canvas: &HtmlCanvasElement| {
                                InMsg::PostBuild(canvas.clone())
                            })),
                    ),
                ),
        )
    }
}


#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Trace).unwrap();

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
        let mut ecs = ecs
            .try_lock()
            .expect("no lock on ecs - request animation loop");
        ecs.maintain();
        ecs.render();
        // We always want to reschedule this animation frame
        true
    });

    App::new(app_ecs).into_component().run()
}
