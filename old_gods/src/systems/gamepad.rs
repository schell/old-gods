//! Joysticks, controllers, etc.
//!
//! The GampadSystem runs the controllers loop, maintaining a map of connected
//! controllers and their state.
//!
//! ## Use:
//! Typically one would use the [PlayerControllers] resource from within another
//! system. Alternatively the [PlayerControllers] resource can be accessed from
//! the world:
//!
//! TODO: #examples of using the GamepadSystem's PlayerControllers resource
//!
//! The controllers maintain what buttons are
//! pressed this frame and last frame, the values of various analog sticks, etc.
//! As well as digital on/off states fore those analog values. Crude timers are
//! used for repeat events, with an initial delay of some integer multiple of
//! the repeat timer.
//!
//! NOTE: All of this is tuned to my preference and should probably be made
//! configurable.
use js_sys::Reflect;
use log::trace;
use specs::prelude::{System, SystemData, World, WorldExt, Write};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{window, Gamepad, GamepadButton};

use super::super::{geom::V2, time::Millis};


/// The analog stick ANALOG_DEADZONE
const ANALOG_DEADZONE: f32 = 0.2;


/// The analog stick repeat cooldown, in millis
const ANALOG_COOLDOWN: u32 = 200;


/// The initial number of cooldowns to wait before repeating.
const FIRST_REPEAT_DELAY_MULTIPLIER: u32 = 3;


#[derive(Debug, Clone, PartialEq)]
pub enum OnMotion {
    OnThisFrame,
    RestingThisFrame,
    RepeatedThisFrame,
}


#[derive(Debug, Clone, PartialEq)]
pub enum OffMotion {
    OffThisFrame,
    RestingThisFrame,
}


#[derive(Debug, Clone, PartialEq)]
pub enum ControllerEventMotion {
    On(OnMotion),
    Off(OffMotion),
}


impl ControllerEventMotion {
    pub fn is_on(&self) -> bool {
        match *self {
            ControllerEventMotion::On(_) => true,
            _ => false,
        }
    }

    pub fn has_repeated_this_frame(&self) -> bool {
        *self == ControllerEventMotion::On(OnMotion::RepeatedThisFrame)
    }

    pub fn is_on_this_frame(&self) -> bool {
        *self == ControllerEventMotion::On(OnMotion::OnThisFrame)
    }

    pub fn is_on_or_repeated_this_frame(&self) -> bool {
        *self == ControllerEventMotion::On(OnMotion::OnThisFrame)
            || *self == ControllerEventMotion::On(OnMotion::RepeatedThisFrame)
    }

    pub fn is_off_this_frame(&self) -> bool {
        *self == ControllerEventMotion::Off(OffMotion::OffThisFrame)
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ControllerEventName {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    X,
    Y,
    Start,
    Back,
}


#[derive(Debug)]
pub struct ControllerEvent {
    pub name: ControllerEventName,
    pub motion: ControllerEventMotion,
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


#[derive(Debug)]
struct ControllerState {
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    start: bool,
    back: bool,
    up: bool,
    up_repeat: bool,
    down: bool,
    down_repeat: bool,
    left: bool,
    left_repeat: bool,
    right: bool,
    right_repeat: bool,
    x_axis: f32,
    y_axis: f32,
}


impl ControllerState {
    fn new() -> Self {
        ControllerState {
            a: false,
            b: false,
            x: false,
            y: false,
            start: false,
            back: false,
            up: false,
            up_repeat: false,
            down: false,
            down_repeat: false,
            left: false,
            left_repeat: false,
            right: false,
            right_repeat: false,
            x_axis: 0.0,
            y_axis: 0.0,
        }
    }
}


/// A player's controller.
/// You can't mess with a player's controller, you can only query it.
#[derive(Debug)]
pub struct PlayerController {
    // If true, the PlayerController just performed some operation taht should disable
    // control for the remainder of the frame.
    debouncing: Cell<bool>,
    controlling_ui: Cell<bool>,
    this_frame: ControllerState,
    last_frame: ControllerState,
    left_above_threshold: Option<(Millis, u32)>,
    right_above_threshold: Option<(Millis, u32)>,
    up_above_threshold: Option<(Millis, u32)>,
    down_above_threshold: Option<(Millis, u32)>,
}


impl PlayerController {
    pub fn new() -> PlayerController {
        PlayerController {
            debouncing: Cell::new(false),
            controlling_ui: Cell::new(false),
            this_frame: ControllerState::new(),
            last_frame: ControllerState::new(),
            left_above_threshold: None,
            right_above_threshold: None,
            up_above_threshold: None,
            down_above_threshold: None,
        }
    }

    /// Debounce the controller, marking it unavailable to systems for the remainder of the
    /// frame.
    pub fn debounce(&self) {
        self.debouncing.set(true);
    }

    /// After calling this the controller will be used to control the UI.
    pub fn use_for_map(&self) {
        if self.controlling_ui.get() {
            self.debounce();
            self.controlling_ui.set(false);
        }
    }

    /// After calling this the controller will be used to control the map.
    pub fn use_for_ui(&self) {
        if !self.controlling_ui.get() {
            self.debounce();
            self.controlling_ui.set(true);
        }
    }

    fn controller_event_motion(this: bool, last: bool, repeat: bool) -> ControllerEventMotion {
        if this {
            ControllerEventMotion::On(
                if last {
                    if repeat {
                        OnMotion::RepeatedThisFrame
                    } else {
                        OnMotion::RestingThisFrame
                    }
                } else {
                    OnMotion::OnThisFrame
                },
            )
        } else {
            ControllerEventMotion::Off(
                if last {
                    OffMotion::OffThisFrame
                } else {
                    OffMotion::RestingThisFrame
                },
            )
        }
    }

    /// Is the player hitting up|down|left|right|...?
    pub fn query(&self, name: &ControllerEventName) -> ControllerEventMotion {
        match name {
            ControllerEventName::A => {
                Self::controller_event_motion(self.this_frame.a, self.last_frame.a, false)
            }
            ControllerEventName::B => {
                Self::controller_event_motion(self.this_frame.b, self.last_frame.b, false)
            }
            ControllerEventName::X => {
                Self::controller_event_motion(self.this_frame.x, self.last_frame.x, false)
            }
            ControllerEventName::Y => {
                Self::controller_event_motion(self.this_frame.y, self.last_frame.y, false)
            }
            ControllerEventName::Start => {
                Self::controller_event_motion(self.this_frame.start, self.last_frame.start, false)
            }
            ControllerEventName::Back => {
                Self::controller_event_motion(self.this_frame.back, self.last_frame.back, false)
            }
            ControllerEventName::Up => Self::controller_event_motion(
                self.this_frame.up,
                self.last_frame.up,
                self.this_frame.up_repeat,
            ),
            ControllerEventName::Down => Self::controller_event_motion(
                self.this_frame.down,
                self.last_frame.down,
                self.this_frame.down_repeat,
            ),
            ControllerEventName::Left => Self::controller_event_motion(
                self.this_frame.left,
                self.last_frame.left,
                self.this_frame.left_repeat,
            ),
            ControllerEventName::Right => Self::controller_event_motion(
                self.this_frame.right,
                self.last_frame.right,
                self.this_frame.right_repeat,
            ),
        }
    }

    pub fn up(&self) -> ControllerEventMotion {
        self.query(&ControllerEventName::Up)
    }

    pub fn down(&self) -> ControllerEventMotion {
        self.query(&ControllerEventName::Down)
    }

    pub fn left(&self) -> ControllerEventMotion {
        self.query(&ControllerEventName::Left)
    }

    pub fn right(&self) -> ControllerEventMotion {
        self.query(&ControllerEventName::Right)
    }

    pub fn a(&self) -> ControllerEventMotion {
        self.query(&ControllerEventName::A)
    }

    pub fn b(&self) -> ControllerEventMotion {
        self.query(&ControllerEventName::B)
    }

    pub fn x(&self) -> ControllerEventMotion {
        self.query(&ControllerEventName::X)
    }

    pub fn y(&self) -> ControllerEventMotion {
        self.query(&ControllerEventName::Y)
    }

    pub fn analog_rate(&self) -> V2 {
        V2::new(
            clear_deadzone(self.this_frame.x_axis),
            clear_deadzone(self.this_frame.y_axis),
        )
    }

    fn step_gamepad(&mut self, gamepad: &Gamepad) {
        self.last_frame = std::mem::replace(&mut self.this_frame, ControllerState::new());

        let now = Millis::now();
        let axes = gamepad.axes();
        if let Some(x) = axes.get(0).as_f64().map(|x| x as f32) {
            self.this_frame.x_axis = x;
            if x > ANALOG_DEADZONE {
                self.this_frame.right = true;
                self.left_above_threshold = None;

                if let Some((last_time, bumps)) = self.right_above_threshold.as_mut() {
                    let millis_since = now.millis_since(*last_time);
                    if millis_since > ANALOG_COOLDOWN {
                        *last_time = now;
                        *bumps += 1;
                        if *bumps > FIRST_REPEAT_DELAY_MULTIPLIER {
                            self.this_frame.right_repeat = true;
                        }
                    }
                } else {
                    self.right_above_threshold = Some((now, 0));
                }
            } else if x < -ANALOG_DEADZONE {
                self.this_frame.left = true;
                self.right_above_threshold = None;

                if let Some((last_time, bumps)) = self.left_above_threshold.as_mut() {
                    let millis_since = now.millis_since(*last_time);
                    if millis_since > ANALOG_COOLDOWN {
                        *last_time = now;
                        *bumps += 1;
                        if *bumps > FIRST_REPEAT_DELAY_MULTIPLIER {
                            self.this_frame.left_repeat = true;
                        }
                    }
                } else {
                    self.left_above_threshold = Some((now, 0));
                }
            } else {
                self.right_above_threshold = None;
                self.left_above_threshold = None;
            }
        }

        if let Some(y) = axes.get(1).as_f64().map(|y| y as f32) {
            self.this_frame.y_axis = y;
            if y > ANALOG_DEADZONE {
                self.this_frame.down = true;
                self.up_above_threshold = None;

                if let Some((last_time, bumps)) = self.down_above_threshold.as_mut() {
                    let millis_since = now.millis_since(*last_time);
                    if millis_since > ANALOG_COOLDOWN {
                        *last_time = now;
                        *bumps += 1;
                        if *bumps > FIRST_REPEAT_DELAY_MULTIPLIER {
                            self.this_frame.down_repeat = true;
                        }
                    }
                } else {
                    self.down_above_threshold = Some((now, 0));
                }
            } else if y < -ANALOG_DEADZONE {
                self.this_frame.up = true;
                self.down_above_threshold = None;

                if let Some((last_time, bumps)) = self.up_above_threshold.as_mut() {
                    let millis_since = now.millis_since(*last_time);
                    if millis_since > ANALOG_COOLDOWN {
                        *last_time = now;
                        *bumps += 1;
                        if *bumps > FIRST_REPEAT_DELAY_MULTIPLIER {
                            self.this_frame.up_repeat = true;
                        }
                    }
                } else {
                    self.up_above_threshold = Some((now, 0));
                }
            }
        }

        let ndx_to_event = |ndx: u32| -> Option<ControllerEventName> {
            match ndx {
                0 => Some(ControllerEventName::A),
                1 => Some(ControllerEventName::B),
                2 => Some(ControllerEventName::X),
                3 => Some(ControllerEventName::Y),
                6 => Some(ControllerEventName::Back),
                7 => Some(ControllerEventName::Start),
                _ => None,
            }
        };

        //let mut msgs = vec![];
        let buttons = gamepad.buttons();
        for btn_ndx in 0..buttons.length() {
            let val: JsValue = buttons.get(btn_ndx);
            match val.dyn_into::<GamepadButton>() {
                Ok(button) => {
                    let pressed = button.pressed();
                    let _value = button.value();
                    if let Some(event) = ndx_to_event(btn_ndx) {
                        match event {
                            ControllerEventName::A => {
                                self.this_frame.a = pressed;
                            }
                            ControllerEventName::B => {
                                self.this_frame.b = pressed;
                            }
                            ControllerEventName::X => {
                                self.this_frame.x = pressed;
                            }
                            ControllerEventName::Y => {
                                self.this_frame.y = pressed;
                            }
                            ControllerEventName::Back => {
                                self.this_frame.back = pressed;
                            }
                            ControllerEventName::Start => {
                                self.this_frame.start = pressed;
                            }
                            _ => {}
                        }
                    } else {
                        //msgs.push(format!("button {} {}", btn_ndx, pressed));
                    }
                }
                Err(_) => panic!("TODO: Support GamepadButton on other browsers"),
            }
        }

        self.debouncing.set(false);
        //if msgs.len() > 0 {
        //  trace!("{}", msgs.join("\n"));
        //}
    }
}


#[derive(Default)]
/// Holds all our player's controller states and other UI related states.
/// This gets queried by various systems that react to the player's input.
pub struct PlayerControllers {
    /// An internal representation of a controller, if available, keyed by its
    /// index.
    controllers: Arc<Mutex<HashMap<u32, PlayerController>>>,

    /// An internal quit var.
    quit_requested: bool,

    /// An internal reload var.
    reload_requested: bool,
}


impl PlayerControllers {
    pub fn should_quit(&self) -> bool {
        self.quit_requested
    }

    pub fn should_reload(&self) -> bool {
        self.reload_requested
    }

    /// Run a function with a player controller only if it is currently
    /// controlling the map.
    pub fn with_map_ctrl_at<F, X>(&self, ndx: u32, f: F) -> Option<X>
    where
        F: FnOnce(&PlayerController) -> X,
    {
        if let Ok(controllers) = self.controllers.try_lock() {
            controllers
                .get(&ndx)
                .map(|ctrl| {
                    if !ctrl.debouncing.get() && !ctrl.controlling_ui.get(){
                        Some(f(ctrl))
                    } else {
                        None
                    }
                })
                .flatten()
        } else {
            None
        }
    }

    /// Run a function with a player controller only if it is currently
    /// controlling the user interface.
    pub fn with_ui_ctrl_at<F, X>(&self, ndx: u32, f: F) -> Option<X>
    where
        F: FnOnce(&PlayerController) -> X,
    {
        if let Ok(controllers) = self.controllers.try_lock() {
            controllers
                .get(&ndx)
                .map(|ctrl| {
                    if !ctrl.debouncing.get() && ctrl.controlling_ui.get(){
                        Some(f(ctrl))
                    } else {
                        None
                    }
                })
                .flatten()
        } else {
            None
        }
    }

}


pub struct GamepadSystem {
    //pub event_pump: EventPump,
    //pub controller_system: GameControllerSubsystem,
    //sdl_controllers: Vec<GameController>,
    web_controllers: Rc<RefCell<HashMap<u32, Gamepad>>>,
}


impl GamepadSystem {
    pub fn new() -> Self {
        GamepadSystem {
            web_controllers: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl<'a> System<'a> for GamepadSystem {
    type SystemData = Write<'a, PlayerControllers>;

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        {
            let pc = world.write_resource::<PlayerControllers>();
            let player_controllers_var = pc.controllers.clone();
            let web_controllers = self.web_controllers.clone();
            let cb = Closure::wrap(Box::new(move |val: JsValue| {
                let gamepad: Gamepad = Reflect::get(&val, &"gamepad".into())
                    .expect("no gamepad")
                    .unchecked_into();
                trace!(
                    "Gamepad connected at index {}: {}. {} buttons, {} axes.",
                    gamepad.index(),
                    gamepad.id(),
                    gamepad.buttons().length(),
                    gamepad.axes().length()
                );

                let mut gamepads = web_controllers.borrow_mut();

                // Get all the connected gamepads and add a player controller for it.
                let mut player_controllers = player_controllers_var
                    .try_lock()
                    .expect("no player controllers lock");
                let window = window().expect("no global window");
                let navigator = window.navigator();
                let nav_gamepads = navigator.get_gamepads().expect("no gamepads array");
                trace!("Found {} available gamepads.", nav_gamepads.length());
                for i in 0..nav_gamepads.length() {
                    let gamepad_val = nav_gamepads.get(i);
                    let gamepad: Gamepad = gamepad_val.unchecked_into();
                    if !player_controllers.contains_key(&gamepad.index()) {
                        player_controllers.insert(gamepad.index(), PlayerController::new());
                    }
                    gamepads.insert(gamepad.index(), gamepad);
                }
            }) as Box<dyn FnMut(JsValue)>);

            window()
                .expect("no global window")
                .add_event_listener_with_callback("gamepadconnected", cb.as_ref().unchecked_ref())
                .expect("could not add gamepadconnected");

            cb.forget();
        }

        {
            let window = window().expect("no global window");
            let pc = world.write_resource::<PlayerControllers>();
            let player_controllers_var = pc.controllers.clone();
            let web_controllers = self.web_controllers.clone();
            let cb = Closure::wrap(Box::new(move |val: JsValue| {
                let gamepad: Gamepad = Reflect::get(&val, &"gamepad".into())
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

                let mut player_controllers = player_controllers_var
                    .try_lock()
                    .expect("no player controllers lock");
                if player_controllers.contains_key(&gamepad.index()) {
                    player_controllers.remove(&gamepad.index());
                }
            }) as Box<dyn FnMut(JsValue)>);

            window
                .add_event_listener_with_callback(
                    "gamepaddisconnected",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("could not add gamepaddisconnected");

            cb.forget();
        }
    }

    fn run(&mut self, ui: Self::SystemData) {
        //// Debug printing
        //let debug_btn = |name: &str, dir: ControllerEventMotion| {
        //  if dir.is_on_this_frame() {
        //    trace!("{:?} on", name);
        //  } else if dir.has_repeated_this_frame() {
        //    trace!("{:?} repeated", name);
        //  } else if dir.is_off_this_frame() {
        //    trace!("{:?} off", name);
        //  }
        //};

        if let Ok(mut controllers) = ui.controllers.try_lock() {
            for (ndx, player_controller) in controllers.iter_mut() {
                let gamepads = self.web_controllers.borrow();
                if let Some(gamepad) = gamepads.get(&ndx) {
                    player_controller.step_gamepad(gamepad);

                //let ctrl = player_controller;
                //debug_btn("left", ctrl.left());
                //debug_btn("right", ctrl.right());
                //debug_btn("up", ctrl.up());
                //debug_btn("down", ctrl.down());
                //debug_btn("a", ctrl.a());
                //debug_btn("b", ctrl.b());
                //debug_btn("x", ctrl.x());
                //debug_btn("y", ctrl.y());
                } else {
                    panic!("no gamepad at index: {}", ndx);
                }
            }
        } else {
            panic!("no lock on controllers!");
        }
    }
}
