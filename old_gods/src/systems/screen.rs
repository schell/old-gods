/// The screen system keeps the players within view of the screen.
use specs::prelude::*;

use super::super::components::{
  Exile, OriginOffset, Player, Position, AABB, V2,
};
use std::f32::{INFINITY, NEG_INFINITY};

#[derive(Debug)]
pub struct Screen {
  /// The screen's aabb in map coordinates.
  map_aabb: AABB,

  /// The width and height of the window.
  pub window_size: (u32, u32),

  /// Width and height of the focus AABB
  tolerance: f32,

  /// Set whether the screen should follow player characters
  pub should_follow_players: bool,
}


impl Screen {
  /// Translate a position to get its relative position within the screen.
  pub fn map_to_screen(&self, pos: &V2) -> V2 {
    *pos - self.aabb().top_left
  }


  /// Translate and scale a position from the screen to the window.
  pub fn screen_to_window(&self, pos: &V2) -> V2 {
    let window_size =
      V2::new(self.window_size.0 as f32, self.window_size.1 as f32);
    let screen_size = self.map_aabb.extents;
    let scale = AABB::scale_needed_to_fit_inside(screen_size, window_size);
    // Move the origin so we're scaling from the center
    let new_size = screen_size.scalar_mul(scale);
    let t = (window_size - new_size).scalar_mul(0.5);
    pos.scalar_mul(scale).translate(&t)
  }


  /// Transform a point in the map to a point in the window, accounting for
  /// screen placement (scale and translation due to aspect fit).
  pub fn map_to_window(&self, p: &V2) -> V2 {
    self.screen_to_window(&self.map_to_screen(p))
  }


  pub fn get_size(&self) -> V2 {
    self.map_aabb.extents
  }


  pub fn set_size(&mut self, (w, h): (u32, u32)) {
    self.map_aabb.extents = V2::new(w as f32, h as f32);
  }


  /// Sets the center of the screen to a map coordinate.
  pub fn set_focus(&mut self, pos: V2) {
    self.map_aabb.set_center(&pos);
  }


  /// Returns the center of the screen in map coordinates.
  pub fn get_focus(&self) -> V2 {
    self.map_aabb.center()
  }


  pub fn get_tolerance(&self) -> f32 {
    self.tolerance
  }


  pub fn focus_aabb(&self) -> AABB {
    let mut aabb = AABB {
      top_left: V2::origin(),
      extents: V2::new(self.tolerance, self.tolerance),
    };
    aabb.set_center(&self.map_aabb.center());
    aabb
  }


  pub fn aabb(&self) -> AABB {
    self.map_aabb
  }


  pub fn distance_to_contain_point(&self, p: &V2) -> V2 {
    let mut out = V2::origin();
    let aabb = self.focus_aabb();
    if p.x < aabb.left() {
      out.x -= aabb.left() - p.x;
    } else if p.x > aabb.right() {
      out.x += p.x - aabb.right();
    }
    if p.y < aabb.top() {
      out.y -= aabb.top() - p.y;
    } else if p.y > aabb.bottom() {
      out.y += p.y - aabb.bottom();
    }

    out
  }
}


impl Default for Screen {
  fn default() -> Screen {
    let aabb = AABB {
      top_left: V2::origin(),
      extents: V2::new(848.0, 648.0),
    };
    Screen {
      map_aabb: aabb,
      window_size: (0, 0),
      tolerance: 50.0,
      should_follow_players: true,
    }
  }
}


pub struct ScreenSystem;


impl<'a> System<'a> for ScreenSystem {
  type SystemData = (
    Entities<'a>,
    ReadStorage<'a, Exile>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, OriginOffset>,
    Write<'a, Screen>,
  );

  fn run(
    &mut self,
    (
      entities,
      exiles,
      players,
      positions,
      offsets,
      mut screen
    ): Self::SystemData,
  ) {
    if !screen.should_follow_players {
      return;
    }
    // First get the minimum bounding rectangle that shows all players.
    let mut left = INFINITY;
    let mut right = NEG_INFINITY;
    let mut top = INFINITY;
    let mut bottom = NEG_INFINITY;

    for (entity, _player, Position(pos), ()) in
      (&entities, &players, &positions, !&exiles).join()
    {
      // TODO: Allow npc players to be counted in screen following.
      // It wouldn't be too hard to have a component ScreenFollows and just search through that,
      // or use this method if there are no ScreenFollows comps in the ECS.
      let offset = offsets.get(entity).map(|o| o.0).unwrap_or(V2::origin());

      let p = *pos + offset;

      if p.x < left {
        left = p.x;
      }
      if p.x > right {
        right = p.x;
      }
      if p.y < top {
        top = p.y;
      }
      if p.y > bottom {
        bottom = p.y;
      }
    }

    let min_aabb = AABB {
      top_left: V2::new(left, top),
      extents: V2::new(right - left, bottom - top),
    };

    let distance = screen.distance_to_contain_point(&min_aabb.center());
    screen.map_aabb.top_left += distance;
  }
}
