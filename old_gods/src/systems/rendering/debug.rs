//use sdl2::rect::Point;
//use sdl2::pixels::Color;
//use sdl2::render::*;
//use sdl2::rect::Rect;
use specs::prelude::*;
use std::collections::HashSet;

use super::super::super::resource_manager::{FontDetails, Sdl2Resources};
use super::super::super::components::*;
use super::super::super::geom::*;
use super::super::super::utils::FPSCounter;
use super::super::physics::*;
use super::super::screen::Screen;
use super::record::*;
use super::text::RenderText;


pub struct RenderDebug;


impl<'ctx, 'res, 'sys> RenderDebug {

  /// Debug draw entity velocities
  fn draw_velocities (
    canvas: &mut WindowCanvas,
    screen: &Screen,
    shapes: &ReadStorage<'sys, Shape>,
    entities: &Entities<'sys>,
    positions: &ReadStorage<'sys, Position>,
    offsets: &ReadStorage<'sys, OriginOffset>,
    velocities: &ReadStorage<'sys, Velocity>
  ) {
    for (entity, position, velo) in (entities, positions, velocities).join() {
      let v  = if velo.0.magnitude() < 1e-10 {
        return;
      } else {
        velo.0
      };
      let offset:V2 =
        entity_local_origin(entity, shapes, offsets);
      let p1 =
        screen
        .from_map(&(position.0 + offset));
      let p2 = p1 + v;
      let lines =
        Self::arrow_lines(p1, p2);
      canvas.set_draw_color(Color::rgb(255, 255, 0));
      canvas.draw_lines(lines.as_slice())
        .expect("Could not draw velocity lines.");
    }
  }


  fn draw_players(
    canvas: &mut WindowCanvas,
    screen: &Screen,
    entities: &Entities<'sys>,
    positions: &ReadStorage<'sys, Position>,
    offsets: &ReadStorage<'sys, OriginOffset>,
    players: &ReadStorage<'sys, Player>,
  ) {
    for (entity, position, _player) in (entities, positions, players).join() {
      let offset = offsets
        .get(entity)
        .map(|o| o.0)
        .unwrap_or(V2::origin());
      let p =
        screen
        .from_map(&(position.0 + offset));
      canvas.set_draw_color(Color::rgb(0, 255, 255));
      canvas.draw_rect(
        Rect::new(
          p.x as i32 - 24,
          p.y as i32 - 24,
          48,
          48
        )
      ).expect("Could not draw player.");
      //let text =
      //  Self::debug_text(format!("{:?}", player));
      //RenderText::draw_text(canvas, resources, &p);
    }
  }


  fn _draw_shape_projection(shape: &Shape, axis: V2, p: V2, map_offset: V2, screen: &Screen, canvas: &mut WindowCanvas) {
    shape
      .vertices()
      .into_iter()
      .for_each(|v| {
        let loc =
          p + v;
        let proj =
          axis.dot(loc);
        print!("{:.2?} ",proj);
        let map_proj_point =
          map_offset + axis.scalar_mul(proj);
        let window_point =
          screen
          .from_map(&map_proj_point);
        // Draw the point itself
        canvas
          .draw_lines(
            Self::point_lines(window_point)
              .as_slice()
          )
          .expect("Could not draw shape point");
      })
  }


  fn draw_map_aabb(aabb: &AABB, screen: &Screen, canvas: &mut WindowCanvas) {
    let dbg_aabb =
      AABB::from_points(
        screen.from_map(&aabb.lower()),
        screen.from_map(&aabb.upper())
      );
    canvas
      .draw_rect(
        dbg_aabb
          .to_rect()
      )
      .expect("Could not draw aabb rect");
  }


  fn draw_map_arrow(from: V2, to: V2, screen: &Screen, canvas: &mut WindowCanvas) {
    let lines =
      Self::arrow_lines(
        screen
          .from_map(&from),
        screen
          .from_map(&to)
      );
    canvas
      .draw_lines(
        lines
          .as_slice()
      )
      .expect("Could not draw map arrow");
  }


  fn draw_map_point(at: V2, screen: &Screen, canvas: &mut WindowCanvas) {
    let lines =
      Self::point_lines(
        screen
          .from_map(&at),
      );
    canvas
      .draw_lines(
        lines
          .as_slice()
      )
      .expect("Could not draw map arrow");
  }



  fn draw_barriers (
    aabb_tree: &AABBTree,
    canvas: &mut WindowCanvas,
    screen: &Screen,
    barriers: &ReadStorage<'sys, Barrier>,
    entities: &Entities<'sys>,
    exiles: &ReadStorage<'sys, Exile>,
    _names: &ReadStorage<'sys, Name>,
    players: &ReadStorage<'sys, Player>,
    positions: &ReadStorage<'sys, Position>,
    shapes: &ReadStorage<'sys, Shape>,
    zlevels: &ReadStorage<'sys, ZLevel>,
    show_collision_info: bool
  ) {
    let player_z:f32 =
      (players, zlevels)
      .join()
      .filter(|(p, _)| p
.0 == 0 )
      .collect::<Vec<_>>()
      .first()
      .clone()
      .map(|(_, z)| z.0)
      .unwrap_or(0.0);

    for (ent, Barrier, shape, Position(p), z) in (entities, barriers, shapes, positions, zlevels).join() {
      let is_exiled = exiles
        .get(ent)
        .map(|_| true)
        .unwrap_or(false);
      let alpha =
        if z.0 == player_z {
          255
        } else {
          50
        };
      let color =
        if is_exiled {
          Color::rgba(255, 255, 255, alpha)
        } else {
          Color::rgba(255, 0, 0, alpha)
        };
      canvas.set_draw_color(color);

      let lines:Vec<Point> =
        shape
        .vertices_closed()
        .into_iter()
        .map(|v| {
          let point =
            screen
            .from_map(&(*p + v));
          Point::new(point.x as i32, point.y as i32)
        })
        .collect();
      canvas
        .draw_lines(
          lines
            .as_slice()
        )
        .expect("Could not draw barrier polygon");

      if show_collision_info {
        // Draw the potential separating axes
        let axes =
          shape
          .potential_separating_axes();
        let midpoints =
          shape
          .midpoints();
        // light red
        let color =
          Color::rgb(255, 128, 128);
        canvas
          .set_draw_color(color);
        for (axis, midpoint) in axes.into_iter().zip(midpoints) {
          let midpoint =
            screen
            .screen_to_viewport(&midpoint);
          let lines =
            Self::arrow_lines(midpoint, midpoint + (axis.scalar_mul(20.0)));
          canvas
            .draw_lines(
              lines
                .as_slice()
            )
            .expect("Could not draw potential separating axis");
        }

        // Draw its collision with other shapes
        let aabb =
          shape
          .aabb()
          .translate(&p);
        let broad_phase_collisions:Vec<(Entity, AABB)> =
          aabb_tree
          .query(&entities, &aabb, &ent);
        broad_phase_collisions
          .into_iter()
          .for_each(|(other_ent, other_aabb)| {
            // Draw the union of their aabbs to show the
            // broad phase collision
            let color =
              Color::rgb(255, 128, 64); // orange
            canvas
              .set_draw_color(color);
            let union =
              AABB::union(&aabb, &other_aabb);
            Self::draw_map_aabb(&union, screen, canvas);

            // Find out if they actually collide and what the
            // mtv is
            let other_shape =
              &shapes
              .get(other_ent)
              .expect("Can't get other shape");
            let other_position =
              positions
              .get(other_ent);
            if other_position.is_none() {
              // This is probably an item that's in an inventory.
              return;
            }
            let other_position =
              other_position
              .unwrap();
            let mtv =
              shape
              .mtv_apart(*p, &other_shape, other_position.0);
            mtv
              .map(|mtv| {
                canvas
                  .set_draw_color(Color::rgb(255, 255, 255));
                Self::draw_map_point(
                  other_aabb.center(),
                  screen,
                  canvas
                );
                Self::draw_map_arrow(
                  other_aabb.center(),
                  other_aabb.center() + mtv,
                  screen,
                  canvas
                );
              });

            //let axes =
            //  other_shape
            //  .potential_separating_axes();
            //axes
            //  .first()
            //  .map(|axis| {
            //    canvas
            //      .set_draw_color(Color::rgb(255, 255, 255));
            //    // Draw the axis
            //    Self::draw_map_arrow(
            //      union.top_left - axis.scalar_mul(15.0),
            //      union.top_left,
            //      screen,
            //      canvas
            //    );
            //    // Draw the projection of the shape's points on the axis
            //    let color =
            //      Color::rgb(128, 128, 255);
            //    canvas
            //      .set_draw_color(color);
            //    Self::draw_shape_projection(&shape, *axis, *p, union.top_left, screen, canvas);
            //    // Draw the projection of the other shape's points on the axis
            //    let color =
            //      Color::rgb(255, 128, 128);
            //    canvas
            //      .set_draw_color(color);
            //    let other_p =
            //      positions
            //      .get(other_ent)
            //      .unwrap()
            //      .0;
            //    Self::draw_shape_projection(&other_shape, *axis, other_p, union.top_left, screen, canvas);
            //    println!("\n\n");
            //  });
          });
      }
    }
  }

  /// Debug rendering
  pub fn draw_debug (
    canvas: &mut WindowCanvas,
    resources: &mut Sdl2Resources<'ctx>,
    toggles: &HashSet<RenderingToggles>,
    aabb_tree: &AABBTree,
    actions: &ReadStorage<'sys, Action>,
    fps: &FPSCounter,
    screen: &Screen,
    entities: &Entities<'sys>,
    names: &ReadStorage<'sys, Name>,
    positions: &ReadStorage<'sys, Position>,
    offsets: &ReadStorage<'sys, OriginOffset>,
    velocities: &ReadStorage<'sys, Velocity>,
    players: &ReadStorage<'sys, Player>,
    barriers: &ReadStorage<'sys, Barrier>,
    shapes: &ReadStorage<'sys, Shape>,
    exiles: &ReadStorage<'sys, Exile>,
    zones: &ReadStorage<'sys, Zone>,
    fences: &ReadStorage<'sys, Fence>,
    step_fences: &ReadStorage<'sys, StepFence>,
    zlevels: &ReadStorage<'sys, ZLevel>
  ) {
    // Get player 0's z
    let player =
      (players, zlevels)
      .join()
      .filter(|(p, _)| p.0 == 0)
      .collect::<Vec<_>>()
      .first()
      .cloned();

    let next_rect =
      if toggles.contains(&RenderingToggles::FPS) {
        let fps = fps.current_fps();
        let fps_text =
          Self::debug_text(format!("FPS:{:.2}", fps).as_str());
        let pos =
          V2::new(0.0, 0.0);
        let rect =
          RenderText::draw_text(canvas, resources, &pos, &fps_text);
        // Unload the text so we don't accumulate a ton of textures
        let _tex =
          resources
          .texture_manager
          .take_resource(&fps_text.as_key())
          .expect("Impossible");
        rect
      } else {
        Rect::new(0, 0, 0, 0)
      };

    if toggles.contains(&RenderingToggles::EntityCount) {
      let count:u32 =
        (entities)
        .join()
        .fold(
          0,
          |n, _| n + 1
        );
      let text =
        Self::debug_text(format!("Entities: {}", count).as_str());
      let pos =
        V2::new(0.0, next_rect.bottom() as f32);
      RenderText::draw_text(canvas, resources, &pos, &text);
    }

    if toggles.contains(&RenderingToggles::Velocities) {
      Self::draw_velocities(
        canvas,
        screen,
        shapes,
        entities,
        positions,
        offsets,
        velocities
      );
    }

    if toggles.contains(&RenderingToggles::AABBTree) {
      let mbrs =
        aabb_tree
        .rtree
        .lookup_in_rectangle(&screen.aabb().to_mbr());
      for EntityBounds{ bounds: mbr, entity_id: id } in mbrs {
        let entity = entities.entity(*id);
        let z =
          zlevels
          .get(entity)
          .or(
            player.map(|p| p.1)
          )
          .cloned()
          .unwrap_or(ZLevel(0.0));
        let alpha =
          if player.is_some() {
            if z.0 == (player.unwrap().1).0 {
              255
            } else {
              50
            }
          } else {
            255
          };
        let color =
          if exiles.contains(entity) {
            Color::rgba(255, 0, 255, alpha)
          } else {
            Color::rgba(255, 255, 0, alpha)
          };
        let aabb =
          AABB::from_mbr(&mbr);
        let aabb =
          AABB::from_points(
            screen
              .from_map(&aabb.top_left),
            screen
              .from_map(&aabb.lower())
          );

        canvas
          .set_draw_color(color);
        canvas
          .draw_rect(
            aabb
              .to_rect()
          ).expect("Could not draw aabb rectangle");
        if let Some(name) = names.get(entity) {
          let p = V2::new(aabb.top_left.x, aabb.bottom());
          let mut text =
            Self::debug_text(name.0.as_str());
          text.color = color;
          RenderText::draw_text(canvas, resources, &p, &text);
        }

      }
    }

    if toggles.contains(&RenderingToggles::Zones) {
      for (entity, Position(p), _, shape) in (entities, positions, zones, shapes).join() {
        let mut color =
          Color::rgb(139, 175, 214);
        let alpha =
          if exiles.contains(entity) {
            128
          } else {
            255
          };
        color.a = alpha;
        let extents =
          shape
          .extents();
        let aabb = AABB::from_points(
          screen.from_map(p),
          screen.from_map(&(*p + extents))
        );
        canvas.set_draw_color(color);
        canvas.draw_rect(
          Rect::new(
            aabb.top_left.x as i32,
            aabb.top_left.y as i32,
            aabb.extents.x as u32,
            aabb.extents.y as u32
          )

        ).expect("Could not draw aabb rectangle");
        if let Some(name) = names.get(entity) {
          let p = V2::new(aabb.top_left.x, aabb.bottom());
          let mut text =
            Self::debug_text(name.0.as_str());
          text.color = color;
          RenderText::draw_text(canvas, resources, &p, &text);
        }
      }
    }

    if toggles.contains(&RenderingToggles::Fences) {
      let aabb =
        screen
        .aabb();
      let filter_fence =
        |p: &Position, points: &Vec<V2> | -> bool {
          for point in points {
            if aabb.contains_point(&(p.0 + *point)) {
              return true
            }
          }
          false
        };
      let mut fences:Vec<(Entity, &Position, &Fence, Color)> =
        (entities, positions, fences)
        .join()
        .filter(|(_,p,f)| {
          filter_fence(p, &f.points)
        })
        .map(|(e,p,f)| (e, p, f, Color::rgb(153, 102, 255)))
        .collect();
      let mut step_fences:Vec<(Entity, &Position, &Fence, Color)> =
        (entities, positions, step_fences)
        .join()
        .filter(|(_, p, f)| {
          filter_fence(p, &f.0.points)
        })
        .map(|(e,p,f)| (e,p,&f.0,Color::rgb(102, 0, 255)))
        .collect();
      fences
        .append(&mut step_fences);

      for (entity, &Position(pos), fence, color) in fences {
        let pos =
          screen
          .from_map(&pos);
        let lines:Vec<Point> =
          fence
          .points
          .iter()
          .map(|p| {
            let p =
              pos + *p;
            Point::new(p.x as i32, p.y as i32)
          })
          .collect();
        canvas.set_draw_color(color);
        canvas.draw_lines(
          lines
            .as_slice()
        ).expect("Could not draw aabb rectangle");
        if let Some(name) = names.get(entity) {
          let text =
            Self::debug_text(name.0.as_str());
          RenderText::draw_text(canvas, resources, &pos, &text);
        }
      }
    }

    //if self.toggles.contains(&RenderingToggles::Positions) {
    //  self.canvas.set_draw_color(Color::rgb(0, 0, 255));
    //  let p = position.0 + *offset;
    //  self.canvas.draw_rect(
    //    Rect::new(
    //      (p.x - focus_offset.x) as i32 - 2,
    //      (p.y - focus_offset.y) as i32 - 2,
    //      4,
    //      4
    //    )
    //  ).expect("Could not draw position.");

    //  let pos_str = format!(
    //    "({}, {}, z{})",
    //    position.0.x.round() as i32,
    //    position.0.y.round() as i32,
    //    may_z.unwrap_or(&ZLevel(0.0)).0
    //  );
    //  self.draw_text(&pos_str, position.0);
    //} else if self.toggles.contains(&RenderingToggles::ZLevels) {
    //  let z = may_z.unwrap_or(&ZLevel(0.0)).0;
    //  self.draw_text(&format!("z{}", z), position.0 - *focus_offset);
    //}

    if toggles.contains(&RenderingToggles::Players)
      && !toggles.contains(&RenderingToggles::Barriers) {
        Self::draw_players(
          canvas,
          screen,
          entities,
          positions,
          offsets,
          players
        );
    }

    if toggles.contains(&RenderingToggles::Screen) {
      canvas.set_draw_color(Color::rgb(0, 255, 0));
      let screen_aabb =
        screen
        .aabb();
      let window_aabb =
        AABB::from_points(
          screen
            .from_map(&screen_aabb.lower()),
          screen
            .from_map(&screen_aabb.upper())
        );

      canvas.draw_rect(
        Rect::new(
          window_aabb.top_left.x as i32,
          window_aabb.top_left.y as i32,
          window_aabb.extents.x as u32,
          window_aabb.extents.y as u32
        )
      ).expect("Could not draw screen border marker.");

      let focus_aabb =
        screen
        .focus_aabb();
      let window_focus_aabb =
        AABB::from_points(
          screen
            .from_map(&focus_aabb.top_left),
          screen
            .from_map(&focus_aabb.lower())
        );
      canvas
        .draw_rect(
          Rect::new(
            window_focus_aabb.top_left.x as i32,
            window_focus_aabb.top_left.y as i32,
            window_focus_aabb.extents.x as u32,
            window_focus_aabb.extents.y as u32
          )
        ).expect("Could not draw screen focus border marker.");
    }

    if toggles.contains(&RenderingToggles::Actions) {
      for (ent, _, Position(pos)) in (entities, actions, positions).join() {
        let is_exiled =
          exiles
          .contains(ent);

        let color = if is_exiled {
          Color::rgb(255, 255, 255)
        } else {
          Color::rgb(252, 240, 5)
        };

        let a =
          screen.from_map(pos);
        let b =
          a + V2::new(10.0, -20.0);
        let c =
          a + V2::new(-10.0, -20.0);
        canvas
          .set_draw_color(color);
        let lines = vec![
          Point::new(a.x as i32, a.y as i32),
          Point::new(b.x as i32, b.y as i32),
          Point::new(c.x as i32, c.y as i32),
          Point::new(a.x as i32, a.y as i32),
        ];
        canvas.draw_lines(lines.as_slice())
          .expect("Could not draw action.");
      }
    }

    if toggles.contains(&RenderingToggles::Shapes) {
      for (shape, Position(p)) in (shapes, positions).join() {
        let color =
          Color::rgb(128, 128, 255);
        canvas
          .set_draw_color(color);

        let lines:Vec<Point> =
          shape
          .vertices_closed()
          .into_iter()
          .map(|v| {
            let point =
              screen
              .from_map(&(*p + v));
            Point::new(point.x as i32, point.y as i32)
          })
          .collect();
        canvas
          .draw_lines(
            lines
              .as_slice()
          )
          .expect("Could not draw shape");
      }
    }

    let show_collision_info =
      toggles.contains(&RenderingToggles::CollisionInfo);
    if toggles.contains(&RenderingToggles::Barriers)
      || show_collision_info {
      Self::draw_barriers(
        aabb_tree,
        canvas,
        screen,
        barriers,
        entities,
        exiles,
        names,
        players,
        positions,
        shapes,
        zlevels,
        show_collision_info
      );
    }
  }
}
