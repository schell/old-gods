/// The screen system keeps the players within view of the screen.
use specs::prelude::*;

use super::super::components::{Exile, OriginOffset, Player, Position, AABB, V2};
use std::f32::{INFINITY, NEG_INFINITY};

/// TODO: Rename to Viewport
#[derive(Debug)]
pub struct Screen {
    /// The screen's aabb in map coordinates.
    viewport: AABB,

    /// Width and height of the focus AABB
    tolerance: f32,

    /// Set whether the screen should follow player characters
    pub should_follow_players: bool,
}


impl Screen {
    /// Translate a position to get its relative position within the screen.
    pub fn from_map(&self, pos: &V2) -> V2 {
        *pos - self.aabb().top_left
    }

    pub fn get_size(&self) -> V2 {
        self.viewport.extents
    }


    pub fn set_size(&mut self, (w, h): (u32, u32)) {
        self.viewport.extents = V2::new(w as f32, h as f32);
    }


    /// Sets the center of the screen to a map coordinate.
    pub fn set_focus(&mut self, pos: V2) {
        self.viewport.set_center(&pos);
    }


    /// Returns the center of the screen in map coordinates.
    pub fn get_focus(&self) -> V2 {
        self.viewport.center()
    }

    /// Returns a mutable viewport.
    pub fn get_mut_viewport(&mut self) -> &mut AABB {
        &mut self.viewport
    }


    pub fn get_tolerance(&self) -> f32 {
        self.tolerance
    }


    pub fn focus_aabb(&self) -> AABB {
        let mut aabb = AABB {
            top_left: V2::origin(),
            extents: V2::new(self.tolerance, self.tolerance),
        };
        aabb.set_center(&self.viewport.center());
        aabb
    }


    pub fn aabb(&self) -> AABB {
        self.viewport
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
            viewport: aabb,
            tolerance: 50.0,
            should_follow_players: true,
        }
    }
}


pub struct ScreenSystem;


#[derive(SystemData)]
pub struct ScreenSystemData<'a> {
    entities: Entities<'a>,
    exiles: ReadStorage<'a, Exile>,
    players: ReadStorage<'a, Player>,
    positions: ReadStorage<'a, Position>,
    offsets: ReadStorage<'a, OriginOffset>,
    screen: Write<'a, Screen>,
}


impl<'a> System<'a> for ScreenSystem {
    type SystemData = ScreenSystemData<'a>;

    fn run(&mut self, mut data: ScreenSystemData) {
        if !data.screen.should_follow_players {
            return;
        }

        // First get the minimum bounding rectangle that shows all players.
        let mut left = INFINITY;
        let mut right = NEG_INFINITY;
        let mut top = INFINITY;
        let mut bottom = NEG_INFINITY;

        for (entity, _player, Position(pos), ()) in (
            &data.entities,
            &data.players,
            &data.positions,
            !&data.exiles,
        )
            .join()
        {
            // TODO: Allow npc players to be counted in screen following.
            // It wouldn't be too hard to have a component ScreenFollows and just search through that,
            // or use this method if there are no ScreenFollows comps in the ECS.
            let offset = data
                .offsets
                .get(entity)
                .map(|o| o.0)
                .unwrap_or_else(V2::origin);

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

        let distance = data.screen.distance_to_contain_point(&min_aabb.center());
        data.screen.viewport.top_left += distance;
    }
}
