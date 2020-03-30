use specs::prelude::*;

//use super::rendering::Rendering;
use super::super::components::{Name, Rendering};
use super::super::utils::FPSCounter;


/// One frame's worth of an Animation
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
  pub rendering: Rendering,
  pub duration: f32,
}


/// A collection of frames, durations and state necessary for
/// animating tiles.
#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
  pub frames: Vec<Frame>,
  pub current_frame_index: usize,
  pub current_frame_progress: f32,
  pub is_playing: bool,
  pub should_repeat: bool,
}

impl Animation {
  pub fn step(&mut self, dt: f32) {
    // Early exit if the animation is not playing.
    if !self.is_playing {
      return;
    }

    self.current_frame_progress += dt;
    'inc_frame: loop {
      if let Some(frame) = self.frames.get(self.current_frame_index) {
        if frame.duration <= self.current_frame_progress {
          self.current_frame_index += 1;
          if self.current_frame_index >= self.frames.len() {
            if self.should_repeat {
              self.current_frame_index = 0;
            } else {
              self.is_playing = false;
              break 'inc_frame;
            }
          }
          self.current_frame_progress -= frame.duration;
        } else {
          break 'inc_frame;
        }
      }
    }
  }


  pub fn get_current_frame(&self) -> Option<&Frame> {
    self.frames.get(self.current_frame_index)
  }


  pub fn stop(&mut self) {
    self.is_playing = false;
  }


  pub fn play(&mut self) {
    self.is_playing = true;
  }


  pub fn seek_to(&mut self, ndx: usize) -> bool {
    if ndx < self.frames.len() {
      self.current_frame_index = ndx;
      true
    } else {
      false
    }
  }

  pub fn has_ended(&self) -> bool {
    self.get_current_frame().is_none()
  }
}


impl Component for Animation {
  type Storage = HashMapStorage<Self>;
}


/// The animation system controls stepping any tiled animations.
pub struct AnimationSystem;

impl<'a> System<'a> for AnimationSystem {
  type SystemData = (
    Read<'a, FPSCounter>,
    Entities<'a>,
    WriteStorage<'a, Animation>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, Rendering>,
  );

  fn run(
    &mut self,
    (fps, entities, mut animation, names, mut renderings): Self::SystemData,
  ) {
    // Find any animations that don't yet have renderings
    let mut frameless_animes = vec![];
    for (ent, ani, _) in (&entities, &animation, !&renderings).join() {
      if let Some(frame) = ani.get_current_frame() {
        // Add the rendering
        println!("Adding rendering for animation {:?}", names.get(ent));
        frameless_animes.push((ent, frame));
      }
    }

    for (e, f) in frameless_animes {
      renderings
        .insert(e, f.rendering.clone())
        .expect("Could not insert rendering for a frameless animation.");
    }

    // Progress any animations.
    for (ani, rndr) in (&mut animation, &mut renderings).join() {
      ani.step(fps.last_delta());
      if let Some(frame) = ani.get_current_frame() {
        *rndr = frame.rendering.clone();
      }
    }
  }
}
