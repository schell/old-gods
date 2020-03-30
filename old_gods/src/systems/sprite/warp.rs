/// The pit is an example of a script subsystem. It relies on the Tiled sprite
/// defined in assets/sprites/zone_pit.json.
use specs::prelude::*;


use super::super::super::components::{
  Animation, Exile, Name, OriginOffset, Position, Script, Sprite, ZLevel, Zone,
};
use super::super::super::geom::V2;


pub enum WarpStep {
  Start,
  End,
}


pub struct WarpGate {
  pub point: Position,
  pub zone: Entity,
  pub effect: Animation,
  pub effect_offset: OriginOffset,
}


pub struct WarpEntry {
  pub entity: Entity,
  pub effect_from: Entity,
  pub effect_to: Entity,
  pub step: WarpStep,
}


impl Component for WarpGate {
  type Storage = HashMapStorage<Self>;
}


impl Component for WarpEntry {
  type Storage = HashMapStorage<Self>;
}


pub struct WarpSystem;


impl WarpSystem {
  pub fn get_warp_point(
    &self,
    names: &ReadStorage<Name>,
    positions: &WriteStorage<Position>,
    sprite: &Sprite,
  ) -> Position {
    for ent in sprite.top_level_children.iter() {
      let name = names.get(*ent);
      if let Some(Name(name)) = name {
        if name == "warp" {
          return positions
            .get(*ent)
            .cloned()
            .expect("WarpSystem is missing a warp point");
        }
      }
    }
    panic!("TheWarp is missing a warp point")
  }

  /// Find the pit zone
  pub fn get_zone_entity(
    &self,
    sprite: &Sprite,
    zones: &ReadStorage<Zone>,
  ) -> Entity {
    for ent in sprite.top_level_children.iter() {
      if zones.contains(*ent) {
        return *ent;
      }
    }
    panic!("WarpGate is missing a zone");
  }

  /// Find the warp effect
  pub fn get_effect(
    &self,
    animations: &WriteStorage<Animation>,
    offsets: &ReadStorage<OriginOffset>,
    sprite: &Sprite,
  ) -> (Entity, Animation, OriginOffset) {
    for ent in sprite.top_level_children.iter() {
      let anime = animations.get(*ent);
      if anime.is_some() {
        let offset = offsets
          .get(*ent)
          .cloned()
          .unwrap_or(OriginOffset(V2::origin()));
        let anime = anime.unwrap().clone();
        return (*ent, anime, offset);
      }
    }
    panic!("TheWarp is missing an animation effect");
  }

  /// Get a new entry to warp
  pub fn get_entry(
    &mut self,
    ent: Entity,
    exiles: &mut WriteStorage<Exile>,
    entities: &Entities,
    from: Position,
    lazy: &LazyUpdate,
    to: Position,
    warpgate: &WarpGate,
    zlevel: ZLevel,
  ) -> WarpEntry {
    // Create an animation at the entity's position
    let mut effect = warpgate.effect.clone();
    effect.should_repeat = false;
    effect.seek_to(0);
    let effect_from = lazy
      .create_entity(entities)
      .with(Name("warp effect from".to_string()))
      .with(effect.clone())
      .with(warpgate.effect_offset.clone())
      .with(from)
      .with(zlevel.clone())
      .build();
    // Stop the effect before cloning it for effect_to
    effect.stop();
    let effect_to = lazy
      .create_entity(entities)
      .with(effect)
      .with(warpgate.effect_offset.clone())
      .with(to)
      .with(zlevel)
      .build();
    // Exile the second effect
    Exile::exile(effect_to, "warpgate", exiles);
    WarpEntry {
      entity: ent,
      effect_from,
      effect_to,
      step: WarpStep::Start,
    }
  }
}


impl<'a> System<'a> for WarpSystem {
  type SystemData = (
    WriteStorage<'a, Animation>,
    Entities<'a>,
    WriteStorage<'a, Exile>,
    Read<'a, LazyUpdate>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, OriginOffset>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, Script>,
    ReadStorage<'a, Sprite>,
    WriteStorage<'a, WarpGate>,
    WriteStorage<'a, WarpEntry>,
    ReadStorage<'a, ZLevel>,
    ReadStorage<'a, Zone>,
  );

  fn run(
    &mut self,
    (
      mut animations,
      entities,
      mut exiles,
      lazy,
      names,
      offsets,
      positions,
      scripts,
      sprites,
      mut warps,
      mut entries,
      zlevels,
      zones,
    ): Self::SystemData,
  ) {
    // First find any scripted sprites that should be turned into warp gates
    for (ent, sprite, script) in (&entities, &sprites, &scripts).join() {
      // Only look for warp scripts
      match script {
        Script::Other { name, .. } => {
          if name != "warp" {
            continue;
          }
        }
        _ => {
          continue;
        }
      }
      // Construct the warpgate
      let zone = self.get_zone_entity(&sprite, &zones);
      let point = self.get_warp_point(&names, &positions, sprite);
      let (effect_ent, effect, effect_offset) =
        self.get_effect(&animations, &offsets, sprite);
      let warpgate = WarpGate {
        zone,
        point,
        effect,
        effect_offset,
      };
      // Exile the original effect
      Exile::exile(effect_ent, "warpgate", &mut exiles);

      let warp_entity = entities.create();
      warps
        .insert(warp_entity, warpgate)
        .expect("Could not insert WarpGate");
      // Remove the scripted sprite as it's not needed
      entities
        .delete(ent)
        .expect("Could not delete a warpgate script sprite");
    }

    // Run through all the warp entries and progress them
    for entry in (&mut entries).join() {
      let effect_from = animations
        .get(entry.effect_from)
        .expect("WarpEntry missing its effect_from");
      if effect_from.has_ended() {
        let effect_to = animations
          .get_mut(entry.effect_to)
          .expect("WarpEntry missing its effect_to");
        if effect_to.has_ended() {
          // Remove the warpee component later
          lazy.remove::<WarpEntry>(entry.entity);
          entities
            .delete(entry.effect_to)
            .expect("Could not delete effect_to");
          entities
            .delete(entry.effect_from)
            .expect("Could not delete effect_from");
        } else if !effect_to.is_playing {
          // Domesticate the to effect
          Exile::domesticate(entry.effect_to, "warpgate", &mut exiles);
          // Domesticate the warpee
          Exile::domesticate(entry.entity, "warpgate", &mut exiles);
          // Exile the from effect
          Exile::exile(entry.effect_from, "warpgate", &mut exiles);
          effect_to.seek_to(0);
          effect_to.play();
        }
      }
    }

    // Run through all the warp gates and add new warp-entries
    for warpgate in (&warps).join() {
      // Get the zone
      let zone = zones
        .get(warpgate.zone)
        .expect("WarpGate's zone is referencing an entity that is not a zone");
      // For all entities that are inside the zone, move them to the warp point
      for ent in zone.inside.iter() {
        if positions.contains(*ent) {
          // Exile it
          Exile::exile(*ent, "warpgate", &mut exiles);
          // Find the entity's position and offset and where it should go
          let entity_position = positions.get(*ent).unwrap().clone();
          let entity_offset = offsets
            .get(*ent)
            .cloned()
            .unwrap_or(OriginOffset(V2::origin()));
          let from_position = Position(
            entity_position.0 + entity_offset.0 - warpgate.effect_offset.0,
          );
          let mut to_position = Position(warpgate.point.0 - entity_offset.0);
          // Move the entity there (the actual "warp")
          lazy.insert(*ent, to_position.clone());
          // The effect just wants to be at the warp point
          to_position.0 = warpgate.point.0 - warpgate.effect_offset.0;
          // Get the entity's zlevel
          let mut zlevel = zlevels.get(*ent).cloned().unwrap_or(ZLevel(0.0));
          zlevel.0 += 1.0;
          // Enter a new entry component for it
          let entry = self.get_entry(
            *ent,
            &mut exiles,
            &entities,
            from_position,
            &lazy,
            to_position,
            &warpgate,
            zlevel,
          );
          entries
            .insert(*ent, entry)
            .expect("Could not insert WarpEntry");
        }
      }
    }
  }
}
