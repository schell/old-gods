//use sdl2::pixels::{Color, PixelFormatEnum};
//use sdl2::render::*;
use specs::prelude::*;
use std::collections::HashSet;
use std::cmp::Ordering;

use super::super::color::Color;
use super::super::components::*;
use super::super::geom::*;
//use super::super::resource_manager::*;
use super::super::utils::FPSCounter;
use super::physics::*;
use super::screen::Screen;
use super::map_loader::Tags;
use super::ui::UI;

pub mod render;

pub mod ui;
use self::ui::RenderUI;

pub mod debug;
use self::debug::RenderDebug;
pub use self::debug::RenderingToggles;

pub mod text;
pub use self::text::RenderText;

mod record;
pub use self::record::*;



// pub struct RenderingSystem {
//   resolution: (u32, u32),
// }
//
//
// impl RenderingSystem {
//
//   /// Create a new rendering system.
//   pub fn new(resolution: (u32, u32)) -> RenderingSystem {
//     RenderingSystem {
//       resolution,
//     }
//   }
//
//
// }
//
// impl<'s> System<'s> for RenderingSystem {
//   type SystemData = (
//     Read<'s, AABBTree>,
//     Read<'s, BackgroundColor>,
//     Read<'s, HashSet<RenderingToggles>>,
//     Write<'s, Screen>,
//     Read<'s, UI>,
//     Read<'s, FPSCounter>,
//     Entities<'s>,
//     ReadStorage<'s, Position>,
//     ReadStorage<'s, Velocity>,
//     ReadStorage<'s, OriginOffset>,
//     ReadStorage<'s, Rendering>,
//     ReadStorage<'s, Barrier>,
//     ReadStorage<'s, ZLevel>,
//     ReadStorage<'s, Exile>,
//     ReadStorage<'s, Item>,
//     ReadStorage<'s, Tags>,
//     ReadStorage<'s, Player>,
//     ReadStorage<'s, Shape>,
//     ReadStorage<'s, Action>,
//     ReadStorage<'s, Name>,
//     ReadStorage<'s, Looting>,
//     ReadStorage<'s, Inventory>,
//     ReadStorage<'s, Item>,
//     ReadStorage<'s, Zone>,
//     ReadStorage<'s, Fence>,
//     ReadStorage<'s, StepFence>,
//   );
//
//   fn run(
//     &mut self, (
//       aabb_tree,
//       background_color,
//       toggles,
//       mut screen,
//       _ui,
//       fps,
//       entities,
//       positions,
//       velo_store,
//       offset_store,
//       renderings,
//       barriers,
//       zlevels,
//       exiles,
//       items,
//       _tag_store,
//       toons,
//       shapes,
//       actions,
//       names,
//       loots,
//       inventories,
//       _items,
//       zones,
//       fences,
//       step_fences
//     ): Self::SystemData
//   ) {
//     // Here are our resources for rendering, we'll pass these around so we must
//     // take them from the RenderingSystem
//     let mut resources =
//       self
//       .resources
//       .take()
//       .expect("RenderingSystem has no resources!");
//
//     let mut target:Texture<'ctx> =
//       if let Some(target) = self.target.take() {
//         target
//       } else {
//         // Create our rendering target
//         let mut tex =
//           resources
//           .texture_creator
//           .create_texture_target(
//             PixelFormatEnum::ABGR8888,
//             self.resolution.0,
//             self.resolution.1
//           )
//           .unwrap();
//         tex
//           .set_blend_mode(BlendMode::Blend);
//         tex
//       };
//
//     let mut canvas =
//       self
//       .canvas
//       .take()
//       .expect("Cannot take a canvas to render to");
//
//     // Set the screen's size and the window size, return the screen's map aabb
//     let screen_aabb = {
//       //screen
//       //  .set_size(self.resolution);
//
//       screen.viewport_size =
//         canvas
//         .output_size()
//         .unwrap();
//
//       screen.aabb()
//     };
//     // Get all the on screen things to render.
//     // Order the things by bottom to top, back to front.
//     let mut ents:Vec<_> = (&entities, &positions, &renderings, !&exiles)
//       .join()
//       .filter(|(_, p, r, _)| {
//         // Make sure we can see this thing (that its destination aabb intersects
//         // the screen)
//         let (w, h) =
//           r.size();
//         let aabb =
//           AABB {
//             top_left: p.0,
//             extents: V2::new(w as f32, h as f32)
//           };
//         screen_aabb.collides_with(&aabb)
//           || aabb.collides_with(&screen_aabb)
//       })
//       .map(|(ent, p, r, _)| {
//         let offset:V2 =
//           entity_local_origin(ent, &shapes, &offset_store);
//         let pos =
//           screen
//           .from_map(&p.0);
//         (ent, Position(pos), offset, r, zlevels.get(ent))
//       })
//       .collect();
//     ents
//       .sort_by( |(_, p1, offset1, _, mz1), (_, p2, offset2, _, mz2)| {
//         let lvl = ZLevel(0.0);
//         let z1 = mz1.unwrap_or(&lvl);
//         let z2 = mz2.unwrap_or(&lvl);
//         if z1.0 < z2.0 {
//           Ordering::Less
//         } else if z1.0 > z2.0 {
//           Ordering::Greater
//         } else if p1.0.y + offset1.y < p2.0.y + offset2.y {
//           Ordering::Less
//         } else if p1.0.y + offset1.y > p2.0.y + offset2.y {
//           Ordering::Greater
//         } else {
//           Ordering::Equal
//         }
//       });
//
//     // Render into our render target texture
//     canvas
//       .with_texture_canvas(&mut target, |mut map_canvas| {
//         map_canvas
//           .set_blend_mode(BlendMode::Blend);
//         map_canvas
//           .set_draw_color(background_color.0);
//         map_canvas
//           .clear();
//
//         // Draw the regular map sprites
//         ents
//           .iter()
//           .for_each(|(_entity, p, _, r, _)| {
//             render::draw_rendering(
//               &mut map_canvas,
//               &mut resources,
//               &p.0,
//               r
//             );
//           });
//
//       })
//       .unwrap();
//
//     // Now use our render target to draw inside the screen, upsampling to the
//     // screen size
//     let src =
//       AABB::new(
//         0.0, 0.0,
//         self.resolution.0 as f32, self.resolution.1 as f32
//       );
//     let dest =
//       AABB::from_points(
//         screen.screen_to_viewport(&src.top_left),
//         screen.screen_to_viewport(&src.extents)
//       )
//       .to_rect();
//     let src =
//       src
//       .to_rect();
//     canvas
//       .set_draw_color(Color::rgb(0, 0, 0));
//     canvas
//       .clear();
//     canvas
//       .copy(&target, Some(src), Some(dest))
//       .unwrap();
//
//     RenderDebug::draw_debug(
//       &mut canvas,
//       &mut resources,
//       &toggles,
//       &aabb_tree,
//       &actions,
//       &fps,
//       &screen,
//       &entities,
//       &names,
//       &positions,
//       &offset_store,
//       &velo_store,
//       &toons,
//       &barriers,
//       &shapes,
//       &exiles,
//       &zones,
//       &fences,
//       &step_fences,
//       &zlevels
//     );
//
//     RenderUI::draw_ui(
//       &mut canvas,
//       &mut resources,
//       &screen,
//       &actions,
//       &entities,
//       &exiles,
//       &inventories,
//       &items,
//       &loots,
//       &names,
//       &offset_store,
//       &positions,
//       &renderings,
//       &toons
//     );
//
//     canvas
//       .present();
//
//     // Give all the things back to ourself
//     self.canvas = Some(canvas);
//     self.target = Some(target);
//     self.resources = Some(resources);
//   }
// }
