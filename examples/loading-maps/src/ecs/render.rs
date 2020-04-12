use old_gods::prelude::*;
use std::{
    cmp::Ordering,
    collections::HashSet,
};

mod action;
mod inventory;

use super::systems::inventory::{Inventory, Loot};


fn fancy_font() -> FontDetails {
    // TODO: Allow the UI font to be customized
    FontDetails {
        path: "monospace".to_string(),
        size: 18,
    }
}


fn fancy_text(msg: &str) -> Text {
    Text {
        text: msg.to_string(),
        font: fancy_font(),
        color: Color::rgb(255, 255, 255),
        size: (16, 16),
    }
}


pub fn normal_font() -> FontDetails {
    // TODO: Allow the UI font to be customized
    FontDetails {
        path: "sans-serif".to_string(),
        size: 16,
    }
}


pub fn normal_text(msg: &str) -> Text {
    Text {
        text: msg.to_string(),
        font: normal_font(),
        color: Color::rgb(255, 255, 255),
        size: (16, 16),
    }
}


#[derive(SystemData)]
struct MapRenderingData<'s> {
    screen: Read<'s, Screen>,
    entities: Entities<'s>,
    positions: ReadStorage<'s, Position>,
    offsets: ReadStorage<'s, OriginOffset>,
    renderings: ReadStorage<'s, Rendering>,
    z_levels: ReadStorage<'s, ZLevel>,
    exiles: ReadStorage<'s, Exile>,
    shapes: ReadStorage<'s, Shape>,
}


pub struct MapEntity {
    pub entity: Entity,
    pub position: V2,
    pub offset: V2,
    pub rendering: Option<Rendering>,
    pub z_level: ZLevel,
}


/// Find all the entities intersecting the visible map.
pub fn get_map_entities(world: &mut World) -> Result<Vec<MapEntity>, String> {
    let data: MapRenderingData = world.system_data();
    let screen_aabb = data.screen.aabb();

    // Get all the on screen things to render.
    // Order the things by bottom to top, back to front.
    let mut ents: Vec<_> = (&data.entities, &data.positions, !&data.exiles)
        .join()
        .filter_map(|(ent, p, ())| {
            // Make sure we can see this thing (that its destination aabb intersects
            // the screen)
            let rendering = data.renderings.get(ent);
            let (w, h) = rendering.map(|r| r.size()).unwrap_or((0, 0));
            let aabb = AABB {
                top_left: p.0,
                extents: V2::new(w as f32, h as f32),
            };
            if !(screen_aabb.collides_with(&aabb) || aabb.collides_with(&screen_aabb)) {
                return None;
            }

            let offset: V2 = entity_local_origin(ent, &data.shapes, &data.offsets);
            let pos = data.screen.from_map(&p.0);
            Some(MapEntity {
                entity: ent,
                position: pos,
                offset,
                rendering: rendering.cloned(),
                z_level: data.z_levels.get(ent).cloned().unwrap_or(ZLevel(0.0)),
            })
        })
        .collect();
    ents.sort_by(|a, b| {
        if a.z_level.0 < b.z_level.0 {
            Ordering::Less
        } else if a.z_level.0 > b.z_level.0 {
            Ordering::Greater
        } else if a.position.y + a.offset.y < b.position.y + b.offset.y {
            Ordering::Less
        } else if a.position.y + a.offset.y > b.position.y + b.offset.y {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    Ok(ents)
}


/// Render the map in a standard way, compositing all of the renderings from back to front, bottom to top.
/// Returns the drawn entities.
pub fn render_map<Ctx: RenderingContext, Rsrc: Resources<Ctx::Image>>(
    world: &mut World,
    resources: &mut Rsrc,
    context: &mut Ctx,
    map_entities: &Vec<MapEntity>,
) -> Result<(), String> {
    let background_color: Read<BackgroundColor> = world.system_data();
    let size = context.context_size()?;
    // Render into our render target texture
    context.set_fill_color(&background_color.0);
    context.fill_rect(&AABB {
        top_left: V2::new(0.0, 0.0),
        extents: V2::new(size.0 as f32, size.1 as f32),
    });
    // Draw the map renderings
    for map_ent in map_entities.iter() {
        if let Some(rendering) = &map_ent.rendering {
            context.draw_rendering(resources, &map_ent.position, &rendering)?;
        }
    }

    Ok(())
}


#[derive(SystemData)]
pub struct DebugRenderingData<'s> {
    aabb_tree: Read<'s, AABBTree>,
    entities: Entities<'s>,
    global_debug_toggles: Read<'s, HashSet<RenderingToggles>>,
    fps: Read<'s, FPSCounter>,
    screen: Read<'s, Screen>,
    velocities: ReadStorage<'s, Velocity>,
    barriers: ReadStorage<'s, Barrier>,
    exiles: ReadStorage<'s, Exile>,
    players: ReadStorage<'s, Player>,
    positions: ReadStorage<'s, Position>,
    object_debug_toggles: ReadStorage<'s, ObjectRenderingToggles>,
    offsets: ReadStorage<'s, OriginOffset>,
    names: ReadStorage<'s, Name>,
    zones: ReadStorage<'s, Zone>,
    fences: ReadStorage<'s, Fence>,
    shapes: ReadStorage<'s, Shape>,
    step_fences: ReadStorage<'s, StepFence>,
    z_levels: ReadStorage<'s, ZLevel>,
}


fn debug_font_details() -> FontDetails {
    FontDetails {
        path: "monospace".to_string(),
        size: 16,
    }
}

fn debug_text(text: &str) -> Text {
    Text {
        text: text.to_string(),
        font: debug_font_details(),
        color: Color::rgb(255, 255, 255),
        size: (16, 16),
    }
}

fn debug_map_text(text: &str) -> Text {
    Text {
        text: text.to_string(),
        font: debug_font_details(),
        color: Color::rgb(255, 255, 255),
        size: (12, 12),
    }
}

/// Construct a vector of lines that form an arrow from p1 to p2
pub fn arrow_lines(p1: V2, p2: V2) -> Vec<V2> {
    let zero = V2::new(0.0, 0.0);
    let n = (p2 - p1).normal().unitize().unwrap_or(zero);
    let p3 = p2 - (p2 - p1).unitize().unwrap_or(zero).scalar_mul(5.0);
    let p4 = p3 + n.scalar_mul(5.0);
    let p5 = p3 - n.scalar_mul(5.0);
    vec![p1, p2, p4, p5, p2]
}

/// Construct a vector of lines that form a kind of hour glass shape.
pub fn point_lines(p: V2) -> Vec<V2> {
    let tl = p + V2::new(-10.0, -10.0);
    let tr = p + V2::new(10.0, -10.0);
    let bl = p + V2::new(-10., 10.0);
    let br = p + V2::new(10.0, 10.0);
    vec![tl.clone(), tr, bl, br, tl]
}


fn draw_map_aabb<Ctx: RenderingContext>(screen: &Screen, context: &mut Ctx) {
    let size = screen.get_size();
    context.stroke_rect(&AABB::new(0.0, 0.0, size.x as f32, size.y as f32));
}


fn draw_map_arrow<Ctx: RenderingContext>(from: V2, to: V2, screen: &Screen, context: &mut Ctx) {
    let lines = arrow_lines(screen.from_map(&from), screen.from_map(&to));
    context.stroke_lines(&lines);
}


fn draw_map_point<Ctx: RenderingContext>(at: V2, screen: &Screen, context: &mut Ctx) {
    let lines = point_lines(screen.from_map(&at));
    context.stroke_lines(&lines);
}


pub fn render_map_debug<Ctx: RenderingContext>(
    world: &mut World,
    context: &mut Ctx,
    map_entities: &Vec<MapEntity>,
) -> Result<(), String> {
    let data: DebugRenderingData = world.system_data();
    let player = (&data.players, &data.z_levels)
        .join()
        .filter(|(p, _)| p.0 == 0)
        .collect::<Vec<_>>()
        .first()
        .cloned();
    for map_ent in map_entities.into_iter() {
        let global_toggles: HashSet<_> = data.global_debug_toggles.clone();
        let obj_toggles: HashSet<_> = data
            .object_debug_toggles
            .get(map_ent.entity)
            .map(|ts| ts.0.clone())
            .unwrap_or(HashSet::new());
        let toggles: HashSet<_> = global_toggles.union(&obj_toggles).collect::<HashSet<_>>();
        render_map_entity_debug(context, &data, toggles, player, map_ent)?;
    }

    Ok(())
}


pub fn draw_velocity<Ctx: RenderingContext>(
    context: &mut Ctx,
    data: &DebugRenderingData,
    map_ent: &MapEntity,
) {
    let velo = if let Some(velo) = data.velocities.get(map_ent.entity) {
        velo
    } else {
        return;
    };

    let v = if velo.0.magnitude() < 1e-10 {
        return;
    } else {
        velo.0
    };
    let offset: V2 = entity_local_origin(map_ent.entity, &data.shapes, &data.offsets);
    let p1 = map_ent.position + offset;
    let p2 = p1 + v;
    let lines = arrow_lines(p1, p2);
    context.set_stroke_color(&Color::rgb(255, 255, 0));
    context.stroke_lines(&lines);
}


pub fn draw_aabb_tree<Ctx: RenderingContext>(
    context: &mut Ctx,
    data: &DebugRenderingData,
    player: &Option<(&Player, &ZLevel)>,
) -> Result<(), String> {
    let mbrs = data
        .aabb_tree
        .rtree
        .lookup_in_rectangle(&data.screen.aabb().to_mbr());
    for EntityBounds {
        bounds: mbr,
        entity_id: id,
    } in mbrs
    {
        let entity = data.entities.entity(*id);
        let z = data
            .z_levels
            .get(entity)
            .or(player.map(|p| p.1))
            .cloned()
            .unwrap_or(ZLevel(0.0));
        let alpha = if player.is_some() {
            if z.0 == (player.unwrap().1).0 {
                255
            } else {
                50
            }
        } else {
            255
        };
        let color = if data.exiles.contains(entity) {
            Color::rgba(255, 0, 255, alpha)
        } else {
            Color::rgba(255, 255, 0, alpha)
        };
        let aabb = AABB::from_mbr(&mbr);
        let aabb = AABB::from_points(
            data.screen.from_map(&aabb.top_left),
            data.screen.from_map(&aabb.lower()),
        );

        context.set_stroke_color(&color);
        context.stroke_rect(&aabb);
        if let Some(name) = data.names.get(entity) {
            let p = V2::new(aabb.top_left.x, aabb.bottom());
            let mut text = debug_text(name.0.as_str());
            text.color = color;
            context.draw_text(&text, &p)?;
        }
    }

    Ok(())
}


pub fn draw_zone<Ctx: RenderingContext>(
    context: &mut Ctx,
    data: &DebugRenderingData,
    map_ent: &MapEntity,
) -> Result<(), String> {
    if let Some(_zone) = data.zones.get(map_ent.entity) {
        if let Some(shape) = data.shapes.get(map_ent.entity) {
            let mut color = Color::rgb(139, 175, 214);
            let alpha = if data.exiles.contains(map_ent.entity) {
                128
            } else {
                255
            };
            color.a = alpha;
            context.set_fill_color(&color);

            let extents = shape.extents();
            let aabb = AABB::from_points(
                data.screen.from_map(&map_ent.position),
                data.screen.from_map(&(map_ent.position + extents)),
            );
            context.fill_rect(&aabb);

            if let Some(name) = data.names.get(map_ent.entity) {
                let p = V2::new(aabb.top_left.x, aabb.bottom());
                let mut text = debug_text(name.0.as_str());
                text.color = color;
                context.draw_text(&text, &p)?;
            }
        }
    }
    Ok(())
}


pub fn draw_fence<Ctx: RenderingContext>(
    context: &mut Ctx,
    data: &DebugRenderingData,
    map_ent: &MapEntity,
) -> Result<(), String> {
    let mut fences = vec![];
    if let Some(fence) = data.fences.get(map_ent.entity) {
        fences.push((fence, Color::rgb(153, 102, 255)));
    }
    if let Some(step_fence) = data.step_fences.get(map_ent.entity) {
        fences.push((&step_fence.0, Color::rgb(102, 0, 255)));
    }

    for (fence, color) in fences {
        let pos = data.screen.from_map(&map_ent.position);
        let lines: Vec<V2> = fence.points.iter().map(|p| pos + *p).collect();
        context.set_fill_color(&color);
        context.stroke_lines(&lines);
        if let Some(name) = data.names.get(map_ent.entity) {
            let text = debug_text(name.0.as_str());
            context.draw_text(&text, &pos)?;
        }
    }

    Ok(())
}

pub fn draw_player<Ctx: RenderingContext>(
    context: &mut Ctx,
    data: &DebugRenderingData,
    map_ent: &MapEntity,
) {
    let p = data.screen.from_map(&(map_ent.position + map_ent.offset));
    context.set_fill_color(&Color::rgb(0, 255, 255));
    context.fill_rect(&AABB::new(p.x - 24.0, p.y - 24.0, 48.0, 48.0));
    //let text =
    //  Self::debug_text(format!("{:?}", player));
    //RenderText::draw_text(canvas, resources, &p);
}

pub fn draw_screen<Ctx: RenderingContext>(context: &mut Ctx, data: &DebugRenderingData) {
    let screen_aabb = data.screen.aabb();
    let window_aabb = AABB::from_points(
        data.screen.from_map(&screen_aabb.lower()),
        data.screen.from_map(&screen_aabb.upper()),
    );
    context.set_stroke_color(&Color::rgb(0, 255, 0));
    context.stroke_rect(&window_aabb);

    let focus_aabb = data.screen.focus_aabb();
    let window_focus_aabb = AABB::from_points(
        data.screen.from_map(&focus_aabb.top_left),
        data.screen.from_map(&focus_aabb.lower()),
    );
    context.stroke_rect(&window_focus_aabb);
}


pub fn draw_action<Ctx: RenderingContext>(
    context: &mut Ctx,
    data: &DebugRenderingData,
    map_ent: &MapEntity,
) {
    let is_exiled = data.exiles.contains(map_ent.entity);

    let color = if is_exiled {
        Color::rgb(255, 255, 255)
    } else {
        Color::rgb(252, 240, 5)
    };

    let a = data.screen.from_map(&map_ent.position);
    let b = a + V2::new(10.0, -20.0);
    let c = a + V2::new(-10.0, -20.0);
    context.set_fill_color(&color);
    let lines = vec![a, b, c, a];
    context.stroke_lines(&lines);
}


pub fn draw_shape<Ctx: RenderingContext>(
    context: &mut Ctx,
    data: &DebugRenderingData,
    map_ent: &MapEntity,
) -> Option<()> {
    let shape = data.shapes.get(map_ent.entity)?;
    let color = Color::rgb(128, 128, 255);
    context.set_fill_color(&color);

    let lines: Vec<V2> = shape
        .vertices_closed()
        .into_iter()
        .map(|v| data.screen.from_map(&(map_ent.position + v)))
        .collect();
    context.stroke_lines(&lines);

    Some(())
}


pub fn draw_barrier<Ctx: RenderingContext>(
    context: &mut Ctx,
    data: &DebugRenderingData,
    show_collision_info: bool,
    player_z: f32,
    map_ent: &MapEntity,
) -> Option<()> {
    let _barrier = data.barriers.get(map_ent.entity)?;
    let shape = data.shapes.get(map_ent.entity)?;
    let z = data.z_levels.get(map_ent.entity)?;
    let is_exiled = data
        .exiles
        .get(map_ent.entity)
        .map(|_| true)
        .unwrap_or(false);
    let alpha = if z.0 == player_z { 255 } else { 50 };
    let color = if is_exiled {
        Color::rgba(255, 255, 255, alpha)
    } else {
        Color::rgba(255, 0, 0, alpha)
    };
    context.set_stroke_color(&color);

    let lines: Vec<V2> = shape
        .vertices_closed()
        .into_iter()
        .map(|v| map_ent.position + v)
        .collect();
    context.stroke_lines(&lines);

    if show_collision_info {
        // Draw the potential separating axes
        let axes = shape.potential_separating_axes();
        let midpoints = shape.midpoints();
        // light red
        let color = Color::rgb(255, 128, 128);
        context.set_stroke_color(&color);
        for (axis, midpoint) in axes.into_iter().zip(midpoints) {
            let lines = arrow_lines(midpoint, midpoint + (axis.scalar_mul(20.0)));
            context.stroke_lines(&lines);
        }

        // Draw its collision with other shapes
        let aabb = shape.aabb().translate(&map_ent.position);
        let broad_phase_collisions: Vec<(Entity, AABB)> =
            data.aabb_tree.query(&data.entities, &aabb, &map_ent.entity);
        broad_phase_collisions
            .into_iter()
            .for_each(|(other_ent, other_aabb)| {
                // Draw the union of their aabbs to show the
                // broad phase collision
                let color = Color::rgb(255, 128, 64); // orange
                context.set_stroke_color(&color);
                draw_map_aabb(&data.screen, context);

                // Find out if they actually collide and what the
                // mtv is
                let other_shape = data.shapes.get(other_ent).expect("Can't get other shape");
                let other_position = data.positions.get(other_ent);
                if other_position.is_none() {
                    // This is probably an item that's in an inventory.
                    return;
                }
                let other_position = other_position.unwrap();
                let mtv = shape.mtv_apart(map_ent.position, &other_shape, other_position.0);
                mtv.map(|mtv| {
                    context.set_stroke_color(&Color::rgb(255, 255, 255));
                    draw_map_point(other_aabb.center(), &data.screen, context);
                    draw_map_arrow(
                        other_aabb.center(),
                        other_aabb.center() + mtv,
                        &data.screen,
                        context,
                    );
                });
            });
    }

    Some(())
}


pub fn draw_position<Ctx: RenderingContext>(
    context: &mut Ctx,
    data: &DebugRenderingData,
    map_ent: &MapEntity,
) -> Result<(), String> {
    context.set_stroke_color(&Color::rgb(0, 0, 255));

    let draw = |label: &str, v: V2, context: &mut Ctx| -> Result<(), String> {
        context.stroke_rect(&AABB::new(v.x as f32 - 2.0, v.y as f32 - 2.0, 4.0, 4.0));

        let pos_str = format!("{}: ({:.1}, {:.1})", label, v.x, v.y,);
        let text = debug_map_text(&pos_str);
        context.draw_text(&text, &v)?;
        Ok(())
    };

    let name = data.names.get(map_ent.entity).map(|Name(n)| n.as_str());
    let pos = "pos";
    let position_label = &name.unwrap_or(pos);
    draw(position_label, map_ent.position, context)?;

    if map_ent.offset != V2::origin() {
        context.set_stroke_color(&Color::rgb(0, 200, 200));
        context.stroke_lines(&arrow_lines(
            map_ent.position,
            map_ent.position + map_ent.offset,
        ));
        draw("orgo", map_ent.position + map_ent.offset, context)?;
    }

    Ok(())
}


pub fn render_map_entity_debug<Ctx: RenderingContext>(
    context: &mut Ctx,
    data: &DebugRenderingData,
    toggles: HashSet<&RenderingToggles>,
    player: Option<(&Player, &ZLevel)>,
    map_ent: &MapEntity,
) -> Result<(), String> {
    if toggles.contains(&RenderingToggles::Positions) {
        draw_position(context, data, map_ent)?;
    }

    if toggles.contains(&RenderingToggles::Velocities) {
        draw_velocity(context, data, map_ent);
    }

    if toggles.contains(&RenderingToggles::Zones) {
        draw_zone(context, data, map_ent)?;
    }

    if toggles.contains(&RenderingToggles::Fences) {
        draw_fence(context, data, map_ent)?;
    }

    if toggles.contains(&RenderingToggles::Players)
        && !toggles.contains(&RenderingToggles::Barriers)
    {
        draw_player(context, data, map_ent);
    }

    if toggles.contains(&RenderingToggles::Actions) {
        draw_action(context, data, map_ent);
    }

    if toggles.contains(&RenderingToggles::Shapes) {
        draw_shape(context, data, map_ent);
    }

    let show_collision_info = toggles.contains(&RenderingToggles::CollisionInfo);
    if toggles.contains(&RenderingToggles::Barriers) || show_collision_info {
        let player_z = player.map(|(_, z)| z.0).unwrap_or(0.0);
        draw_barrier(context, data, show_collision_info, player_z, map_ent);
    }

    Ok(())
}


type UIRenderingData<'s> = (
    Entities<'s>,
    Read<'s, Screen>,
    ReadStorage<'s, Action>,
    ReadStorage<'s, Exile>,
    ReadStorage<'s, Inventory>,
    ReadStorage<'s, Loot>,
    ReadStorage<'s, Name>,
    ReadStorage<'s, OriginOffset>,
    ReadStorage<'s, Player>,
    ReadStorage<'s, Position>,
    ReadStorage<'s, Shape>,
);


pub fn render_ui<Ctx:RenderingContext, Rsrc:Resources<Ctx::Image>>(
    world: &mut World,
    resources: &mut Rsrc,
    context: &mut Ctx,
    // The function needed to convert a point in the map viewport to the context.
    viewport_to_context: impl Fn(V2) -> V2,
) -> Result<(), String> {
    let (
        _entities,
        screen,
        actions,
        exiles,
        inventories,
        loots,
        names,
        origin_offsets,
        players,
        positions,
        shapes,
    ): UIRenderingData = world.system_data();

    for (action, ()) in (&actions, !&exiles).join() {
        // Only render actions if they have a player that is elligible.
        for elligible_ent in action.elligibles.iter() {
            if players.contains(*elligible_ent) {
                if let Some(position) = positions.get(*elligible_ent) {
                    let offset = entity_local_origin(*elligible_ent, &shapes, &origin_offsets);
                    let extra_y_offset = shapes
                        .get(*elligible_ent)
                        .map(|s| s.extents() * V2::new(-0.5, 0.5) + V2::new(0.0, 4.0))
                        .unwrap_or(V2::origin());
                    let point = position.0 + offset + extra_y_offset;
                    let point = viewport_to_context(screen.from_map(&point));
                    action::draw(context, &point, action)?;
                }
            }
        }
    }

    // Draw lootings involving a player that are on the screen
    for (loot, _) in (&loots, !&exiles).join() {
        let has_position = positions.contains(loot.looter)
            || (loot.inventory.is_some() && positions.contains(loot.inventory.unwrap()));
        let has_player = players.contains(loot.looter)
            || (loot.inventory.is_some() && players.contains(loot.inventory.unwrap()));
        if !has_position || !has_player {
            continue;
        }
        let mut players_vec = vec![players.get(loot.looter).cloned()];
        loot.inventory.map(|i| {
            let player = players.get(i).cloned();
            players_vec.push(player);
        });
        let players_vec: Vec<Player> = players_vec.into_iter().filter_map(|t| t).collect();
        let may_player: Option<&Player> = players_vec.first();
        if may_player.is_some() {
            let loot_rendering = inventory::make_loot_rendering(&loot, &inventories, &names);
            inventory::draw_loot(context, resources, &V2::new(10.0, 10.0), loot_rendering)?;
        }
    }

    Ok(())
}

/// Renders debug user interface.
pub fn render_ui_debug<Ctx:RenderingContext>(
    world: &mut World,
    context: &mut Ctx,
    // The function needed to convert a point in the map viewport to the context.
    _viewport_to_context: impl Fn(V2) -> V2,
) -> Result<(), String> {
    let data: DebugRenderingData = world.system_data();
    let next_rect = if data.global_debug_toggles.contains(&RenderingToggles::FPS) {
        let fps_text = debug_text(&data.fps.current_fps_string());
        let pos = V2::new(0.0, 10.0);
        context.draw_text(&fps_text, &pos)?;
        let size = context.measure_text(&fps_text)?;

        // Draw a graph of the FPS
        {
            let averages = data.fps.second_averages();
            let max_average = averages.iter().fold(0.0, |a, b| f32::max(a, *b));
            let mut x = pos.x + size.0;
            let height = size.1 + 10.0;
            let y = (pos.y + height).round();
            let mut points = vec![
                V2::new(pos.x + size.0 + FPS_COUNTER_BUFFER_SIZE as f32, y),
                V2::new(pos.x + size.0, y),
            ];
            for avg in averages.into_iter() {
                let percent = avg / max_average;
                points.push(V2::new(x, y - (percent * height)));
                x += 1.0
            }
            context.set_stroke_color(&old_gods::color::css::gold());
            context.stroke_lines(&points);
        }

        AABB {
            top_left: pos,
            extents: V2 {
                x: size.0,
                y: size.1,
            },
        }
    } else {
        AABB::identity()
    };

    let toggles = &data.global_debug_toggles;

    if toggles.contains(&RenderingToggles::EntityCount) {
        let count: u32 = (&data.entities).join().fold(0, |n, _| n + 1);
        let text = debug_text(format!("Entities: {}", count).as_str());
        let pos = V2::new(0.0, next_rect.bottom() as f32 + 10.0);
        context.draw_text(&text, &pos)?;
    }

    if toggles.contains(&RenderingToggles::AABBTree) {
        let player = (&data.players, &data.z_levels)
            .join()
            .filter(|(p, _)| p.0 == 0)
            .collect::<Vec<_>>()
            .first()
            .cloned();
        draw_aabb_tree(context, &data, &player)?;
    }

    if toggles.contains(&RenderingToggles::Screen) {
        draw_screen(context, &data);
    }

    Ok(())
}
