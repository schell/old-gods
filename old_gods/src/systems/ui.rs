/// The UI system runs the event loop, maintaining a map of connected controllers
/// and their state. The controllers maintain what buttons are pressed this
/// frame, the values of various analog stickes, etc. As well as digital on/off
/// states fore those analog values.
use js_sys::Reflect;
use log::trace;
use specs::prelude::{
  System,
  SystemData,
  World,
  Write
};
use std::collections::HashMap;
use std::time::Instant;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{
  closure::Closure,
  JsCast,
  JsValue,
  UnwrapThrowExt
};
use web_sys::{
  Gamepad,
  window,
};


use super::super::components::Cardinal;
//use sdl2::event::Event;
//use sdl2::keyboard::{Keycode, Mod};
//use sdl2::EventPump;
//use sdl2::GameControllerSubsystem;
//use sdl2::controller::{Button, Axis, GameController};

use super::super::geom::V2;


/// The analog stick ANALOG_DEADZONE
const ANALOG_DEADZONE: f32 = 0.2;

/// The analog stick repeat cooldown, in millis
const ANALOG_COOLDOWN: u128 = 170;


#[derive(Debug, Clone)]
pub enum OnMotion {
  OnThisFrame,
  RestingThisFrame,
  RepeatedThisFrame
}


#[derive(Debug, Clone)]
pub enum OffMotion {
  OffThisFrame,
  RestingThisFrame
}


#[derive(Debug, Clone)]
pub enum ControllerEventMotion {
  On(OnMotion), Off(OffMotion)
}


impl ControllerEventMotion {
  pub fn is_on(&self) -> bool {
    match *self {
      ControllerEventMotion::On(_) => true,
      _ => false
    }
  }

  pub fn has_repeated_this_frame(&self) -> bool {
    match *self {
      ControllerEventMotion::On(OnMotion::RepeatedThisFrame) => true,
      _ => false
    }
  }

  pub fn is_on_this_frame(&self) -> bool {
    match *self {
      ControllerEventMotion::On(OnMotion::OnThisFrame) => true,
      _ => false
    }
  }

  pub fn is_off_this_frame(&self) -> bool {
    match *self {
      ControllerEventMotion::Off(OffMotion::OffThisFrame) => true,
      _ => false
    }
  }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ControllerEventName {
  Up, Down, Left, Right,
  A, B, X, Y,
  Start, Back,
}


impl ControllerEventName {
  //pub fn from_keycode(keycode: &Keycode) -> Option<ControllerEventName> {
  //  use ControllerEventName::*;
  //  match keycode {
  //    Keycode::Up => {Some(Up)}
  //    Keycode::Down => {Some(Down)}
  //    Keycode::Left => {Some(Left)}
  //    Keycode::Right => {Some(Right)}
  //    Keycode::A => {Some(A)}
  //    Keycode::B => {Some(B)}
  //    Keycode::X => {Some(X)}
  //    Keycode::Y => {Some(Y)}
  //    _ => None
  //  }
  //}
}


#[derive(Debug)]
pub struct ControllerEvent {
  pub name: ControllerEventName,
  pub motion: ControllerEventMotion
}


pub fn scale_i16(i: i16) -> f32 {
  if i >= 0 {
    i as f32 / i16::max_value() as f32
  } else {
    -1.0 * (i as f32 / i16::min_value() as f32)
  }
}


pub fn clear_deadzone(f: f32) -> f32 {
  if f.abs() > ANALOG_DEADZONE {
    f
  } else {
    0.0
  }
}


/// A player's controller.
/// You can't mess with a player's controller, you can only query it.
#[derive(Debug)]
pub struct PlayerController {
  last_seen_events: HashMap<ControllerEventName, ControllerEventMotion>,
  events: Vec<ControllerEvent>,
  left_analog: V2,
  left_above_threshold: Option<Instant>,
  right_above_threshold: Option<Instant>,
  up_above_threshold: Option<Instant>,
  down_above_threshold: Option<Instant>,
}


impl PlayerController {
  pub fn new() -> PlayerController {
    PlayerController {
      last_seen_events: HashMap::new(),
      events: vec![],
      left_analog: V2::new(0.0, 0.0),
      left_above_threshold: None,
      right_above_threshold: None,
      up_above_threshold: None,
      down_above_threshold: None,
    }
  }

  /// Is the player hitting up|down|left|right|...?
  pub fn query (&self, name: &ControllerEventName) -> ControllerEventMotion {
    let may_this_frame =
      self
      .events
      .iter()
      .filter(|ev| ev.name == *name)
      .collect::<Vec<_>>()
      .first()
      .map(|ev| ev.motion.clone());
    let may_last_frame =
      self
      .last_seen_events
      .get(&name);
    match (may_this_frame, may_last_frame) {
      (None, None) => {
        ControllerEventMotion::Off(OffMotion::RestingThisFrame)
      }

      (Some(motion_now), _) => {
        motion_now.clone()
      }

      (None, Some(motion_last)) => {
        if motion_last.is_on() {
          ControllerEventMotion::On(OnMotion::RestingThisFrame)
        } else {
          ControllerEventMotion::Off(OffMotion::RestingThisFrame)
        }
      }
    }
  }

  pub fn up (&self) -> ControllerEventMotion {
    self.query(&ControllerEventName::Up)
  }

  pub fn down (&self) -> ControllerEventMotion {
    self.query(&ControllerEventName::Down)
  }

  pub fn left (&self) -> ControllerEventMotion {
    self.query(&ControllerEventName::Left)
  }

  pub fn right (&self) -> ControllerEventMotion {
    self.query(&ControllerEventName::Right)
  }

  pub fn a (&self) -> ControllerEventMotion {
    self.query(&ControllerEventName::A)
  }

  pub fn b (&self) -> ControllerEventMotion  {
    self.query(&ControllerEventName::B)
  }

  pub fn x (&self) -> ControllerEventMotion  {
    self.query(&ControllerEventName::X)
  }

  pub fn y (&self) -> ControllerEventMotion {
    self.query(&ControllerEventName::Y)
  }

  pub fn analog_rate(&self) -> V2 {
    V2::new(
      clear_deadzone(self.left_analog.x),
      clear_deadzone(self.left_analog.y)
    )
  }

  ///// Step the player's controller.
  //fn step(&mut self, sdl_controller: &GameController) {
  //  self.events = vec![];
  //  let x = scale_i16(sdl_controller.axis(Axis::LeftX));
  //  let y = scale_i16(sdl_controller.axis(Axis::LeftY));
  //  self.update_axis(true, x);
  //  self.update_axis(false, y);
  //}

  fn add_event(&mut self, name: ControllerEventName, motion: ControllerEventMotion) {
    self.events.push(ControllerEvent {
      name: name.clone(),
      motion: motion.clone()
    });
    self.last_seen_events.insert(name, motion);
  }

  fn axis_threshold(&self, dir: &Cardinal) -> Option<Instant> {
    match dir {
      Cardinal::East => {
        self.left_above_threshold.clone()
      }
      Cardinal::West => {
        self.right_above_threshold.clone()
      }
      Cardinal::North => {
        self.up_above_threshold.clone()
      }
      Cardinal::South => {
        self.down_above_threshold.clone()
      }
    }
  }

  fn axis_threshold_mut(&mut self, dir: &Cardinal) -> &mut Option<Instant> {
    match dir {
      Cardinal::East => {
        &mut self.left_above_threshold
      }
      Cardinal::West => {
        &mut self.right_above_threshold
      }
      Cardinal::North => {
        &mut self.up_above_threshold
      }
      Cardinal::South => {
        &mut self.down_above_threshold
      }
    }
  }

  fn axis_event_name(&self, dir: &Cardinal) -> ControllerEventName {
    match dir {
      Cardinal::East => {
        ControllerEventName::Left
      }
      Cardinal::West => {
        ControllerEventName::Right
      }
      Cardinal::North => {
        ControllerEventName::Up
      }
      Cardinal::South => {
        ControllerEventName::Down
      }
    }
  }

  fn update_dir(&mut self, dir: &Cardinal, beyond_deadzone: bool) {
    let now = Instant::now();
    let this_event = self.axis_event_name(dir);
    let this_threshold_in = self.axis_threshold(dir);
    let may_motion =
      if beyond_deadzone {
        if let Some(last) = this_threshold_in {
          if now.duration_since(last).as_millis() > ANALOG_COOLDOWN {
            Some(ControllerEventMotion::On(OnMotion::RepeatedThisFrame))
          } else {
            None
          }
        } else {
          Some(ControllerEventMotion::On(OnMotion::OnThisFrame))
        }
      } else {
        if this_threshold_in.is_some() {
          Some(ControllerEventMotion::Off(OffMotion::OffThisFrame))
        } else {
          None
        }
      };

    if let Some(motion) = may_motion {
      self.add_event(this_event, motion.clone());

      let this_threshold = self.axis_threshold_mut(dir);
      *this_threshold =
        if motion.is_on() {
          Some(now)
        } else {
          None
        }
    }
  }

  fn update_axis(&mut self, is_horizontal: bool, value: f32) {
    let dir =
      if is_horizontal {
        self.left_analog.x = value;
        Cardinal::West
      } else {
        self.left_analog.y = value;
        Cardinal::South
      };
    let on_dir =
      if value > 0.0 {
        dir
      } else {
        dir.opposite()
      };
    let off_dir = on_dir.opposite();
    let beyond_deadzone = value.abs() > ANALOG_DEADZONE;
    self.update_dir(&on_dir, beyond_deadzone);
    self.update_dir(&off_dir, false);
  }
}


#[derive(Default)]
/// Holds all our player's controller states and other UI related states.
/// This gets read by various systems that react to the player's input.
pub struct UI {
  /// An internal representation of a controller, if available, keyed by its
  /// index in sdl2's list of GameControllers.
  controllers: HashMap<u32, PlayerController>,

  /// An internal quit var.
  quit_requested: bool,

  /// An internal reload var.
  reload_requested: bool
}


impl UI {
  pub fn should_quit(&self) -> bool {
    self.quit_requested
  }

  pub fn should_reload(&self) -> bool {
    self.reload_requested
  }

  pub fn get_player_controller(&self, ndx: u32) -> Option<&PlayerController> {
    self.controllers.get(&ndx)
  }
}


pub struct UISystem {
  //pub event_pump: EventPump,
  //pub controller_system: GameControllerSubsystem,
  //sdl_controllers: Vec<GameController>,
  web_controllers: Rc<RefCell<HashMap<u32, Gamepad>>>
}


impl UISystem {
  pub fn new() -> Self {
    UISystem{
      web_controllers: Rc::new(RefCell::new(HashMap::new()))
    }
  }
  ///// Add a controller to the system and UI resource by sdl controller index.
  ///// This inits/opens the controller, in sdl2 terms.
  //pub fn add_controller(&mut self, ndx: u32, ui: &mut UI) {
  //  let cont = self
  //    .controller_system
  //    .open(ndx.clone())
  //    .expect("Could not open controller.");
  //  let name = cont.name();
  //  println!("Opened controller {}", name);
  //  self.sdl_controllers.push(cont);
  //  ui.controllers.insert(ndx, PlayerController::new());
  //}

  //pub fn new(
  //  controller_system: GameControllerSubsystem,
  //  event_pump: EventPump,
  //) -> UISystem {
  //  UISystem {
  //    controller_system,
  //    event_pump,
  //    sdl_controllers: vec![]
  //  }
  //}

  //pub fn sdl2_btn_to_event(btn: &Button) -> Option<ControllerEventName> {
  //  match btn {
  //    Button::A => {
  //      Some(ControllerEventName::A)
  //    }
  //    Button::B => {
  //      Some(ControllerEventName::B)
  //    }
  //    Button::X => {
  //      Some(ControllerEventName::X)
  //    }
  //    Button::Y => {
  //      Some(ControllerEventName::Y)
  //    }
  //    Button::Start => {
  //      Some(ControllerEventName::Start)
  //    }
  //    Button::Back => {
  //      Some(ControllerEventName::Back)
  //    }
  //    n => {
  //      println!("Unsupported button '{:?}'", n);
  //      None
  //    }
  //  }
  //}
}


impl<'a> System<'a> for UISystem {
  type SystemData = Write<'a, UI>;

  //fn setup(&mut self, res: &mut Resources) {
  //  Self::SystemData::setup(res);

  //  // Make sure we add any controllers that are available at startup
  //  let available = match self.controller_system.num_joysticks() {
  //    Ok(n) => n,
  //    Err(e) => panic!("Can't enumerate joytsicks: {}", e),
  //  };
  //  println!("{} joysticks available at UISystem startup", available);

  //  let mut ui = res.fetch_mut();
  //  for ndx in 0 .. available {
  //    self.add_controller(ndx, &mut ui);
  //  }
  //}

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);

    let window =
      window()
      .expect("no global window");

    {
      let web_controllers = self.web_controllers.clone();
      let cb = Closure::wrap(Box::new(move |val:JsValue| {
        let gamepad:Gamepad =
          Reflect::get(&val, &"gamepad".into())
          .expect("no gamepad")
          .dyn_into()
          .expect("cant coerce gamepad");
        trace!(
          "Gamepad connected at index {}: {}. {} buttons, {} axes.",
          gamepad.index(),
          gamepad.id(),
          gamepad.buttons().length(),
          gamepad.axes().length()
        );

        let mut gamepads = web_controllers.borrow_mut();
        gamepads.insert(gamepad.index(), gamepad);
      }) as Box<dyn FnMut(JsValue)>);

      window
        .add_event_listener_with_callback("gamepadconnected", cb.as_ref().unchecked_ref())
        .unwrap_throw();

      cb.forget();
    }

    {
      let web_controllers = self.web_controllers.clone();
      let cb = Closure::wrap(Box::new(move |val:JsValue| {
        let gamepad:Gamepad =
          Reflect::get(&val, &"gamepad".into())
          .expect("no gamepad")
          .dyn_into()
          .expect("cant coerce gamepad");
        trace!(
          "Gamepad disconnected at index {}: {}. {} buttons, {} axes.",
          gamepad.index(),
          gamepad.id(),
          gamepad.buttons().length(),
          gamepad.axes().length()
        );

        let mut gamepads = web_controllers.borrow_mut();
        gamepads.remove(&gamepad.index());
      }) as Box<dyn FnMut(JsValue)>);

      window
        .add_event_listener_with_callback("gamepaddisconnected", cb.as_ref().unchecked_ref())
        .unwrap_throw();

      cb.forget();
    }

    // TODO: Add more controller and keyboard events to the UISystem.
    //      match event {
    //    // Key events for Player(0)
    //    Event::KeyDown { keycode: Some(k), repeat, ..} => {
    //      if let Some(ctrl) = ui.controllers.get_mut(&0) {
    //        if let Some(name) = ControllerEventName::from_keycode(&k) {
    //          let motion = if repeat {
    //            ControllerEventMotion::On(OnMotion::RepeatedThisFrame)
    //          } else {
    //            ControllerEventMotion::On(OnMotion::OnThisFrame)
    //          };
    //          ctrl.add_event(name, motion);
    //        }
    //      }
    //    }

    //    Event::KeyUp { keycode: Some(k),  ..} => {
    //      if let Some(ctrl) = ui.controllers.get_mut(&0) {
    //        if let Some(name) = ControllerEventName::from_keycode(&k) {
    //          let motion = ControllerEventMotion::Off(OffMotion::OffThisFrame);
    //          ctrl.add_event(name, motion);
    //        }
    //      }
    //    }

    //    Event::ControllerButtonUp { which, button, .. } => {
    //      if let Some(ctrl) = ui.controllers.get_mut(&(which as u32)) {
    //        let motion = ControllerEventMotion::Off(OffMotion::OffThisFrame);
    //        Self::sdl2_btn_to_event(&button)
    //          .map(|ev| {
    //            ctrl.add_event(ev, motion);
    //          });
    //      }
    //    }

    //    Event::ControllerButtonDown { which, button, .. } => {
    //      if let Some(ctrl) = ui.controllers.get_mut(&(which as u32)) {
    //        let motion = ControllerEventMotion::On(OnMotion::OnThisFrame);
    //        Self::sdl2_btn_to_event(&button)
    //          .map(|ev| {
    //            ctrl.add_event(ev, motion);
    //          });
    //      }
    //    }

    //    Event::ControllerAxisMotion { which, axis, value, .. } => {
    //      let scaled_value = scale_i16(value);
    //      if let Some(ctrl) = ui.controllers.get_mut(&(which as u32)) {
    //        match axis {
    //          Axis::LeftX => {
    //            ctrl.update_axis(true, scaled_value);
    //          }
    //          Axis::LeftY => {
    //            ctrl.update_axis(false, scaled_value);
    //          }
    //          _ => {
    //            println!("Unsupported axis:{:?}",axis);
    //          }
    //        }
    //      }
    //    }

    //    _ev => {
    //      //println!("Unsupported event:\n{:?}", ev);
    //    }
    //  }
    //}
  }

  fn run (&mut self, _ui: Self::SystemData) {
  }


  //fn run(&mut self, mut ui: Self::SystemData) {
  //  // Reset some of the UI's state
  //  ui.quit_requested = false;
  //  ui.reload_requested = false;

  //  // Step over all player controllers and update their internal state
  //  for (ndx, controller) in ui.controllers.iter_mut() {
  //    let sdl_controller =
  //      self
  //      .sdl_controllers
  //      .get(ndx.clone() as usize)
  //      .expect(&format!("Could not find sdl controller {}", ndx));
  //    controller.step(sdl_controller);
  //  }

  //  // Step over sdl events.
  //  let mut added_controllers = vec![];
  //  for event in self.event_pump.poll_iter() {
  //    //println!("event:\n{:?}\n", event);
  //    match event {
  //      // If the user quits the window (hit the X) or hits escape, we leave.
  //      Event::Quit {..} => {
  //        ui.quit_requested = true;
  //      }

  //      Event::KeyDown { keycode: Some(Keycode::R), .. } => {
  //        ui.reload_requested = true;
  //      }

  //      Event::ControllerDeviceAdded{ timestamp:_, which: ndx } => {
  //        println!("Adding controller device {}", ndx);
  //        added_controllers.push(ndx);
  //      }

  //      Event::KeyDown { keycode: Some(Keycode::Q), keymod, ..} => {
  //        if keymod.contains(Mod::LCTRLMOD)
  //          || keymod.contains(Mod::RCTRLMOD) {
  //            ui.quit_requested = true;
  //          }
  //      }

  //      // Key events for Player(0)
  //      Event::KeyDown { keycode: Some(k), repeat, ..} => {
  //        if let Some(ctrl) = ui.controllers.get_mut(&0) {
  //          if let Some(name) = ControllerEventName::from_keycode(&k) {
  //            let motion = if repeat {
  //              ControllerEventMotion::On(OnMotion::RepeatedThisFrame)
  //            } else {
  //              ControllerEventMotion::On(OnMotion::OnThisFrame)
  //            };
  //            ctrl.add_event(name, motion);
  //          }
  //        }
  //      }

  //      Event::KeyUp { keycode: Some(k),  ..} => {
  //        if let Some(ctrl) = ui.controllers.get_mut(&0) {
  //          if let Some(name) = ControllerEventName::from_keycode(&k) {
  //            let motion = ControllerEventMotion::Off(OffMotion::OffThisFrame);
  //            ctrl.add_event(name, motion);
  //          }
  //        }
  //      }

  //      Event::ControllerButtonUp { which, button, .. } => {
  //        if let Some(ctrl) = ui.controllers.get_mut(&(which as u32)) {
  //          let motion = ControllerEventMotion::Off(OffMotion::OffThisFrame);
  //          Self::sdl2_btn_to_event(&button)
  //            .map(|ev| {
  //              ctrl.add_event(ev, motion);
  //            });
  //        }
  //      }

  //      Event::ControllerButtonDown { which, button, .. } => {
  //        if let Some(ctrl) = ui.controllers.get_mut(&(which as u32)) {
  //          let motion = ControllerEventMotion::On(OnMotion::OnThisFrame);
  //          Self::sdl2_btn_to_event(&button)
  //            .map(|ev| {
  //              ctrl.add_event(ev, motion);
  //            });
  //        }
  //      }

  //      Event::ControllerAxisMotion { which, axis, value, .. } => {
  //        let scaled_value = scale_i16(value);
  //        if let Some(ctrl) = ui.controllers.get_mut(&(which as u32)) {
  //          match axis {
  //            Axis::LeftX => {
  //              ctrl.update_axis(true, scaled_value);
  //            }
  //            Axis::LeftY => {
  //              ctrl.update_axis(false, scaled_value);
  //            }
  //            _ => {
  //              println!("Unsupported axis:{:?}",axis);
  //            }
  //          }
  //        }
  //      }

  //      _ev => {
  //        //println!("Unsupported event:\n{:?}", ev);
  //      }
  //    }
  //  }

  //  // Add the controllers pushed in the event pump
  //  for ndx in added_controllers {
  //    self.add_controller(ndx, &mut ui);
  //  }

  //  // Debug printing
  //  //let debug_btn = | name: &str, dir: ControllerEventMotion | {
  //  //  if dir.is_on_this_frame() {
  //  //    println!("{:?} on", name);
  //  //  } else if dir.has_repeated_this_frame() {
  //  //    println!("{:?} repeated", name);
  //  //  } else if dir.is_off_this_frame() {
  //  //    println!("{:?} off", name);
  //  //  }
  //  //};
  //  //if let Some(ctrl) = ui.controllers.get_mut(&0) {
  //  //  debug_btn("left", ctrl.left());
  //  //  debug_btn("right", ctrl.right());
  //  //  debug_btn("up", ctrl.up());
  //  //  debug_btn("down", ctrl.down());
  //  //  debug_btn("a", ctrl.a());
  //  //  debug_btn("b", ctrl.b());
  //  //  debug_btn("x", ctrl.x());
  //  //  debug_btn("y", ctrl.y());
  //  //}
  //}
}
