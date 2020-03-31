/// The TweenSystem applies affine transformations to entities over time. Be careful, it will
/// overwrite your component values if that's the thing its tweening.
use specs::prelude::*;

use super::super::prelude::{Position, Velocity, V2};
use super::super::time::FPSCounter;


/// The thing that's being tweened.
#[derive(Debug, Clone)]
pub enum TweenParam {
  Position(V2, V2),
  Velocity(V2, V2),
}


/// The easing function being used to tween a subject.
#[derive(Debug, Clone)]
pub enum Easing {
  Linear,
}


impl Easing {
  /// t b c d
  /// `t` is current time
  /// `b` is the start value
  /// `c` is the total change in value
  /// `d` is the duration
  pub fn tween(&self, t: f32, b: f32, c: f32, d: f32) -> f32 {
    match self {
      Easing::Linear => c * t / d + b,
    }
  }
}


#[derive(Debug, Clone)]
pub struct Tween {
  pub subject: Entity,
  pub param: TweenParam,
  pub easing: Easing,
  pub dt: f32,
  pub duration: f32,
}


impl Component for Tween {
  type Storage = HashMapStorage<Self>;
}


/// Helper function for tweening an entity.
pub fn tween(
  entities: &Entities,
  subject: Entity,
  lazy: &LazyUpdate,
  param: TweenParam,
  easing: Easing,
  duration: f32, // in seconds
) {
  let _ = lazy
    .create_entity(&entities)
    .with(Tween {
      subject,
      param,
      easing,
      duration,
      dt: 0.0,
    })
    .build();
}


pub struct TweenSystem;


impl<'a> System<'a> for TweenSystem {
  type SystemData = (
    Entities<'a>,
    Write<'a, FPSCounter>,
    Read<'a, LazyUpdate>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Tween>,
    WriteStorage<'a, Velocity>,
  );

  fn run(
    &mut self,
    (
      entities,
      fps,
      lazy,
      mut positions,
      mut tweens,
      mut velocities
    ): Self::SystemData,
  ) {
    for (ent, mut tween) in (&entities, &mut tweens).join() {
      let delta = fps.last_delta();
      tween.dt += delta;

      let tween_is_dead = tween.dt > tween.duration;
      if tween_is_dead {
        // This tween is done, remove it
        lazy.remove::<Tween>(ent);
      }

      let tween_v2 = |v: &mut V2, start: V2, end: V2| {
        if tween_is_dead {
          *v = end;
        } else {
          v.x = tween.easing.tween(
            tween.dt,
            start.x,
            end.x - start.x,
            tween.duration,
          );
          v.y = tween.easing.tween(
            tween.dt,
            start.y,
            end.y - start.y,
            tween.duration,
          );
        }
      };

      match tween.param.clone() {
        TweenParam::Position(start, end) => {
          tween_v2(
            &mut positions
              .get_mut(tween.subject)
              .expect("Trying to tween an entity without a position")
              .0,
            start,
            end,
          );
        }
        TweenParam::Velocity(start, end) => {
          tween_v2(
            &mut velocities
              .get_mut(tween.subject)
              .expect("Trying to tween an entity without a velocity")
              .0,
            start,
            end,
          );
        }
      }
    }
  }
}
