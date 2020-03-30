extern crate either;
extern crate nom;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate shred;
extern crate shrev;
extern crate spade;
extern crate specs;

pub mod color;
pub mod engine;
pub mod tiled;
pub mod systems;
//pub mod resource_manager;
pub mod components;
pub mod geom;
pub mod prelude;
pub mod utils;
pub mod parser;
pub mod time;

//use sdl2::Sdl;
//use sdl2::video::WindowContext;
//use sdl2::render::WindowCanvas;
//use sdl2::pixels::Color;
//use sdl2::render::TextureCreator;
//use sdl2::ttf::Sdl2TtfContext;
//use specs::prelude::*;
//use shrev::EventChannel;

//use prelude::*;


// /// The context the engine is running in. On desktop this is a wrapper around
// /// sdl2. At some point this may turn into a trait for cross platform concerns.
// pub struct EngineContext;
//
//
// impl EngineContext {
//   /// Create a new context (window), set the title, size, etc.
//   pub fn new_ctx(
//     title: &str,
//     (ww, wh): (u32, u32)
//   ) -> (Sdl, WindowCanvas, TextureCreator<WindowContext>, Sdl2TtfContext) {
//     let ctx =
//       sdl2::init()
//       .expect("Could not create sdl2 context.");
//     let vsys =
//       ctx
//       .video()
//       .expect("Could not init video system.");
//     let window =
//       vsys
//       .window(title, ww, wh)
//       .position_centered()
//       .resizable()
//       .build()
//       .expect("Could not create a window.");
//     let mut canvas =
//       window
//       .into_canvas()
//       .build()
//       .expect("Could not create a canvas.");
//     canvas.set_draw_color(Color::rgb(0, 0, 0));
//     canvas.clear();
//     canvas.present();
//     let ttf_ctx =
//       sdl2::ttf::init()
//       .unwrap();
//     let texture_creator =
//       canvas.texture_creator();
//     (ctx, canvas, texture_creator, ttf_ctx)
//   }
// }


// /// Used to end the main loop. If a system fetches this resources and sets its
// /// value to `true`, the main loop will break.
// pub struct ExitEngine(pub bool);
//
//
// impl Default for ExitEngine {
//   fn default() -> Self {
//     ExitEngine(false)
//   }
// }


// /// Start the engine.
// /// This opens a window, reads the Tiled map given, sets everything up end
// /// runs the main loop until the user quits or a system sets the ExitEngine bit.
// ///
// /// If `resolution` is `None`, the resolution will be that of the loaded Tiled
// /// map, but if `file` is `None`, the resolution will default to (800, 600).
// // TODO: engine::run should take a HashSet of config options
// pub fn run<Ctx, F, G, X>(
//   ctx: &mut Ctx,
//   file: Option<&str>,
//   may_dispatcher_builder: Option<DispatcherBuilder>,
//   setup_fun: F,
//   exit_fun: G
// ) -> Option<X>
// where
//   Ctx: EngineContext,
//   F: FnOnce(&mut World) + 'static + Send + Sync,
//   G: FnOnce(&mut World) -> X + 'static
// {
//   // Create the world
//   let mut world = World::new();
//   world
//     .setup::<Read<ExitEngine>>();
//
//   // Create the renderer
//   let mut rendering_system =
//     RenderingSystem::new(
//       canvas,
//       resources,
//       (800, 600)
//     );
//   <RenderingSystem as System>::SystemData::setup(&mut world.res);
//
//
//   let mut dispatcher = {
//     let event_pump =
//       ctx
//       .event_pump()
//       .expect("Could not pump events.");
//     let controller_system =
//       ctx
//       .game_controller()
//       .expect("Could not init controller system");
//     may_dispatcher_builder
//       .unwrap_or(DispatcherBuilder::new())
//       .with_thread_local(GamepadSystem::new(controller_system, event_pump))
//       .with_thread_local(SoundSystem::new())
//       .with(MapLoadingSystem{ opt_reader: None }, "map", &[])
//       .with(ScreenSystem, "screen", &[])
//       .with(ActionSystem, "action", &[])
//       .with(ScriptSystem, "script", &["action"])
//       .with(SpriteSystem, "sprite", &["script"])
//       .with(PlayerSystem, "control", &[])
//       .with(Physics::new(), "physics", &[])
//       .with(AnimationSystem, "animation", &[])
//       .with(InventorySystem, "inventory", &[])
//       .with(EffectSystem, "effect", &[])
//       .with(ItemSystem, "item", &["action", "effect"])
//       .with(ZoneSystem, "zone", &[])
//       .with(WarpSystem, "warp", &["physics"])
//       .with(FenceSystem, "fence", &["physics"])
//       .with(TweenSystem, "tween", &[])
//       .build()
//   };
//
//   dispatcher
//     .setup(&mut world.res);
//
//   // Maintain once so all our resources are created.
//   world
//     .maintain();
//
//   // Give the library user a chance to set up their world
//   setup_fun(&mut world);
//
//   // Load our map file
//   file
//     .map(|file| {
//       world
//         .write_resource::<EventChannel<MapLoadingEvent>>()
//         .single_write(MapLoadingEvent::LoadMap(file.to_string(), V2::new(0.0, 0.0)));
//     });
//
//   let exit;
//
//   'running: loop {
//     {
//       // Update the delta (and FPS)
//       world
//         .write_resource::<FPSCounter>()
//         .next_frame();
//     }
//
//     dispatcher
//       .dispatch(&mut world.res);
//     world
//       .maintain();
//
//     if world.read_resource::<UI>().should_reload() {
//       let mut map_chan =
//         world
//         .write_resource::<EventChannel<MapLoadingEvent>>();
//       map_chan
//         .single_write(MapLoadingEvent::UnloadEverything);
//       file
//         .map(|file| {
//           map_chan
//             .single_write(MapLoadingEvent::LoadMap(
//               file.to_string(),
//               V2::new(0.0, 0.0)
//             ));
//         });
//     }
//
//     // Check if the UI asked us to quit
//     if world.read_resource::<UI>().should_quit() {
//       exit = None;
//       break 'running;
//     } else if world.read_resource::<ExitEngine>().0 {
//       exit = Some(exit_fun(&mut world));
//       break 'running;
//     }
//
//     // Render everything
//     let data:<RenderingSystem as System>::SystemData =
//       SystemData::fetch(&world.res);
//     rendering_system
//       .run(data);
//   }
//   exit
// }
