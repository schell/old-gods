/// Attributes allow us to read components and entities out of a Tiled map.
/// This module provides some shared functionality for other */record.rs files.
use either::Either;
use log::trace;
use serde_json::Value;
use specs::prelude::{
  Builder, Component, DenseVecStorage, Entity, World, WorldExt,
};
use std::collections::HashMap;

use super::super::parser::hex_color;
use super::super::prelude::{
  find_by,
  get_tile_animation,
  get_tile_rendering,
  object_barrier,
  object_shape,
  Action,
  Animation,
  Barrier,
  CanBeEmpty,
  Color,
  Fence,
  FitnessStrategy,
  FontDetails,
  GlobalTileIndex,
  Inventory,
  Item,
  Lifespan,
  MaxSpeed,
  //Music,
  Name,
  Object,
  ObjectGroup,
  OriginOffset,
  Player,
  Point,
  Position,
  Property,
  Rendering,
  Shape,
  //Sound,
  StepFence,
  Text,
  TextValue,
  Tiledmap,
  ZLevel,
  Zone,
  //Trigger,
  V2,
};


/// An enumeration of attributes that many entities may have.
#[derive(Debug, Clone)]
pub enum Attribute {
  Action(Action),
  Barrier(Shape),
  Player(Player),
  Fence(Fence),
  StepFence(StepFence),
  Inventory(String),
  Item(Item),
  //Lifespan(Lifespan),
  MaxSpeed(MaxSpeed),
  //Music(Music),
  Name(Name),
  OriginOffset(OriginOffset),
  Position(Position),
  RenderingOrAnime(Either<Rendering, Animation>),
  Shape(Shape),
  //Sound(Sound),
  ZIncrement(i32),
  Zone(Shape),
  //JSON(JSON)
}


impl Attribute {
  /// Scale an attribute by an amount in X and Y.
  /// This is used to adjust barriers and origins on scaled Tiled objects.
  // TODO: Support for flipped tile objects
  pub fn into_scaled(self, scale: &V2) -> Attribute {
    match self {
      Attribute::Barrier(s) => Attribute::Barrier(s.into_scaled(scale)),
      Attribute::OriginOffset(o) => {
        let o = OriginOffset(o.0 * *scale);
        Attribute::OriginOffset(o)
      }
      att => att,
    }
  }
}


/// A collection of attributes with some convenience functions.
#[derive(Debug, Clone)]
pub struct Attributes {
  pub attribs: Vec<Attribute>,
}


impl Attributes {
  //pub fn read_sound(obj: &Object) -> Result<Sound, String> {
  //  let file =
  //    obj
  //    .properties
  //    .get("file")
  //    .ok_or("A sound must have a 'file' property")?
  //    .to_string();
  //  let volume:f32 =
  //    obj
  //    .properties
  //    .get("volume")
  //    .map(|s| {
  //      from_str(s)
  //        .map_err(|e| format!("{}",e))
  //    })
  //    .unwrap_or(
  //      Ok(1.0)
  //    )?;
  //  let trigger:Trigger =
  //    obj
  //    .properties
  //    .get("trigger")
  //    .map(|s| {
  //      match s.as_str() {
  //        "loop" => { Ok(Trigger::Loop) }
  //        "once" => { Ok(Trigger::Once) }
  //        s => { Err(format!("{:?} is not a valid sound trigger value.", s)) }
  //      }
  //    })
  //    .unwrap_or(Err("Sound must have a 'trigger' property".to_string()))?;

  //  Ok(Sound{
  //    file,
  //    volume,
  //    trigger,
  //    channel: None
  //  })
  //}

  /// Read an object as one attribute. The object should have a valid value for
  /// its 'Type' property.
  /// This includes attribute objects like:
  /// * Action
  /// * Barrier
  /// * OriginOffset
  /// * Fence
  /// * StepFence
  /// TODO: Put all the Tiled object parsing in one place.
  pub fn read_single_attribute(obj: &Object) -> Result<Attribute, String> {
    let properties = obj
      .properties
      .iter()
      .map(|prop| (prop.name.clone(), prop.value.clone()))
      .collect::<HashMap<_, _>>();
    match obj.type_is.as_str() {
      "item" => {
        let usable: bool = properties
          .get("usable")
          .map(|value: &Value| {
            value
              .as_bool()
              .ok_or("item's 'usable' property must be bool")
          })
          .unwrap_or(Ok(false))?;
        let stack: Option<usize> = {
          if let Some(stack_value) = properties.get("stack_count") {
            let num = stack_value.as_u64().ok_or(
              "items's 'stack_count' property must be unsigned int".to_string(),
            )?;
            Some(num as usize)
          } else {
            None
          }
        };
        let item = Item { usable, stack };
        Ok(Attribute::Item(item))
      }

      "origin_offset" => {
        let p = V2::new(obj.x, obj.y);
        Ok(Attribute::OriginOffset(OriginOffset(p)))
      }

      "barrier" => {
        let shape = object_barrier(obj)
          .ok_or(&format!("Invalid barrier type.\n{:?}", obj))?;
        Ok(Attribute::Barrier(shape))
      }

      "action" => {
        let text_value: &Value = properties
          .get("text")
          .ok_or("An action must have a 'text' property")?;
        let text: String = text_value
          .as_str()
          .ok_or("An action's 'text' property must be a string")?
          .to_string();
        let strategy_val: &Value = properties
          .get(&FitnessStrategy::tiled_key())
          .ok_or("An action must have a 'fitness' property")?;
        let strategy_str = strategy_val
          .as_str()
          .ok_or("An action's 'fitness' property must be a string")?;
        let strategy =
          FitnessStrategy::from_str(strategy_str).map_err(|e| {
            format!("Could not parse action's fitness strategy: {:?}", e)
          })?;
        let lifespan_value: &Value = properties
          .get("lifespan")
          .ok_or("An action must have a 'lifespan' property")?;

        let lifespan: Lifespan = if let Some(lifespan_str) =
          lifespan_value.as_str()
        {
          match lifespan_str {
            "forever" => Lifespan::Forever,
            s => {
              return Err(format!(
                "Lifespan value must be the string \"forever\" or an int. Found '{}'",
                s
              ));
            }
          }
        } else {
          let num =
            lifespan_value
            .as_u64()
            .ok_or(
              format!(
                "Lifespan value must be the string \"forever\" or an int. Found {:#?}",
                lifespan_value
              )
            )? as u32;
          Lifespan::Many(num)
        };

        let action = Action {
          elligibles: vec![],
          taken_by: vec![],
          text,
          strategy,
          lifespan,
        };
        Ok(Attribute::Action(action))
      }
      "zone" => {
        let shape =
          object_shape(obj).ok_or("Zone does not have a valid shape")?;
        Ok(Attribute::Zone(shape))
      }
      "fence" => {
        let polyline: Vec<Point<f32>> = obj
          .polyline
          .clone()
          .ok_or("Encoutered a fence that is not a polyline")?;
        let points: Vec<V2> = polyline
          .into_iter()
          .map(|Point { x, y }| V2::new(x, y))
          .collect::<Vec<_>>();
        Ok(Attribute::Fence(Fence::new(points)))
      }
      "step_fence" => {
        let polyline: Vec<Point<f32>> = obj
          .polyline
          .clone()
          .ok_or("Encoutered a fence that is not a polyline")?;
        let points: Vec<V2> = polyline
          .into_iter()
          .map(|Point { x, y }| V2::new(x, y))
          .collect::<Vec<_>>();
        Ok(Attribute::StepFence(StepFence(Fence::new(points))))
      }
      //"sound" => {
      //  let sound =
      //    Self::read_sound(obj)?;
      //  Ok(Attribute::Sound(sound))
      //}
      //"music" => {
      //  let sound =
      //    Self::read_sound(obj)?;
      //  Ok(Attribute::Music(Music(sound)))
      //}
      att => Err(format!("Unsupported single attribute object {}", att)),
    }
  }


  /// Read a number of attributes from a hashmap of properties.
  pub fn read_properties(
    properties: &HashMap<&String, &Property>,
  ) -> Result<Vec<Attribute>, String> {
    let mut attribs = vec![];

    // Player
    if let Some(control_scheme_prop) = properties.get(&Player::tiled_key()) {
      let control_scheme = control_scheme_prop
        .value
        .as_str()
        .ok_or("player object's control_scheme must be a string".to_string())?;
      let control = match control_scheme.as_ref() {
        "player" => {
          let ndx_prop =
            properties.get(&"player_index".to_string()).ok_or({
              trace!("properties:\n{:#?}", properties);
              "Object must have a 'player_index' custom property for control."
                .to_string()
            })?;
          let ndx = ndx_prop.value.as_u64().ok_or(
            "Object 'player_index' must be an unsigned integer: {:?}"
              .to_string(),
          )?;
          Player(ndx as u32)
        }

        "npc" => {
          panic!("TODO: support NPCs");
        }

        _ => {
          panic!("Unsupported control scheme '{}'.", control_scheme);
        }
      };
      attribs.push(Attribute::Player(control));
    }

    // ZIncrement
    if let Some(z_inc) = properties
      .get(&"z_inc".to_string())
      .map(|p| p.value.as_i64())
      .flatten()
    {
      attribs.push(Attribute::ZIncrement(z_inc as i32));
    }

    // MaxSpeed
    if let Some(max_speed) = properties
      .get(&MaxSpeed::tiled_key())
      .map(|p| p.value.as_f64())
      .flatten()
    {
      attribs.push(Attribute::MaxSpeed(MaxSpeed(max_speed as f32)));
    }

    // Inventory
    if let Some(p) = properties.get(&Inventory::tiled_key_name()) {
      if let Some(name) = p.value.as_str() {
        attribs.push(Attribute::Inventory(name.to_string()));
      }
    }

    //// Script
    //if let Some(script) = properties
    //  .get(&Script::tiled_key())
    //  .map(|p| p.value.as_str().map(|s| s.to_string()))
    //  .flatten()
    //{
    //  let mut property_map = HashMap::new();
    //  for prop in properties.values() {
    //    property_map.insert(prop.name.clone(), prop.value.clone());
    //  }
    //  let script = Script::from_str(&script, Some(property_map))?;
    //  attribs.push(Attribute::Script(script));
    //}

    Ok(attribs)
  }

  /// Read a number of attributes from a tiled tile's GlobalTileIndex.
  pub fn read_gid(
    map: &Tiledmap,
    gid: &GlobalTileIndex,
    size: Option<(u32, u32)>,
  ) -> Result<Vec<Attribute>, String> {
    let mut attribs = vec![];

    // RenderingOrAnime
    let anime = get_tile_animation(&map, gid, size).map(|a| Either::Right(a));
    let rend = get_tile_rendering(&map, gid, size).map(|r| Either::Left(r));
    if let Some(rendering_or_anime) = anime.or(rend) {
      attribs.push(Attribute::RenderingOrAnime(rendering_or_anime));
    }

    let scale = Attributes {
      attribs: attribs.clone(),
    }
    .scale();

    if let Some(tile) = map.get_tile(&gid.id) {
      type MyResult = Result<Vec<Attribute>, String>;
      let mut single_attribs: Vec<Attribute> = tile.object_group.iter().fold(
        Ok(vec![]),
        |res: MyResult, group: &ObjectGroup| -> MyResult {
          let mut res_atts = res?;
          res_atts.append(&mut group.objects.iter().fold(
            Ok(vec![]),
            |res_atts: Result<Vec<Attribute>, String>, obj| {
              let mut atts = res_atts?;
              let att = Attributes::read_single_attribute(&obj)?;
              let att = att.into_scaled(&scale);
              println!("Got nested single object attribute:\n{:?}", att);
              atts.push(att);
              Ok(atts)
            },
          )?);
          Ok(res_atts)
        },
      )?;
      attribs.append(&mut single_attribs);
    }
    Ok(attribs)
  }

  /// Read a Text rendering from the object
  pub fn read_as_text(object: &Object) -> Result<Attribute, String> {
    let text = object
      .text
      .get("text")
      .ok_or("Tiled text is missing its text property")?
      .get_string()
      .ok_or("Tiled text 'text' property is not a string")?;
    let color: Color = object
      .text
      .get("color")
      .map(|tv| {
        let s: String = tv
          .get_string()
          .ok_or("Tiled text 'color' property is not a color string")?;
        hex_color(s.as_str())
          .map_err(|e| format!("{:?}", e))
          .map(|(_, c)| c)
      })
      .unwrap_or(Ok(Color::rgb(0, 0, 0)))?;
    let font_family = object
      .text
      .get("fontfamily")
      .cloned()
      .unwrap_or(TextValue::String("sans-serif".to_string()))
      .get_string()
      .ok_or("Tiled text 'fontfamily' property is not a string")?;
    let size = object
      .text
      .get("pixelsize")
      .map(|tv: &TextValue| -> Result<u16, String> {
        let sz = tv
          .get_uint()
          .ok_or("Tiled text 'pixelsize' property is not a uint")?;
        Ok(sz)
      })
      .unwrap_or(Ok(16))?;
    let font = FontDetails {
      path: font_family,
      size,
    };
    let size = (
      f32::round(object.width) as u32,
      f32::round(object.height) as u32,
    );
    let text = Text {
      text,
      color,
      font,
      size,
    };
    Ok(Attribute::RenderingOrAnime(Either::Left(
      Rendering::from_text(text),
    )))
  }

  /// Read a number of attributes from a tiled Object.
  pub fn read(map: &Tiledmap, object: &Object) -> Result<Attributes, String> {
    let mut attributes = Attributes { attribs: vec![] };

    // Position
    // Tiled tiles' origin are at the bottom of the tile, not the top
    let y = object.y - object.height;
    let p = V2::new(object.x, y);
    attributes.attribs.push(Attribute::Position(Position(p)));

    if let Some(name) = object.name.non_empty() {
      attributes.attribs.push(Attribute::Name(Name(name.clone())));
    }

    if let Some(shape) = object_shape(object) {
      attributes
        .attribs
        .push(Attribute::Shape(shape.translated(&p.scalar_mul(-1.0))));
    }

    let object_props = object
      .properties
      .iter()
      .map(|p| (&p.name, p))
      .collect::<HashMap<_, _>>();
    let mut object_property_attribs = Self::read_properties(&object_props)?;
    attributes.attribs.append(&mut object_property_attribs);

    let mut tile_property_attribs = if let Some(gid) = &object.gid {
      if let Some(tile) = map.get_tile(&gid.id) {
        let tile_props = tile
          .properties
          .iter()
          .map(|p| (&p.name, p))
          .collect::<HashMap<_, _>>();
        Self::read_properties(&tile_props)?
      } else {
        vec![]
      }
    } else {
      vec![]
    };
    attributes.attribs.append(&mut tile_property_attribs);

    // Any single object type
    if let Ok(att) = Self::read_single_attribute(object) {
      attributes.attribs.push(att);
    }

    let nested_attribs: Result<Vec<Attribute>, String> =
      if let Some(gid) = &object.gid {
        let size = (object.width as u32, object.height as u32);
        Attributes::read_gid(map, gid, Some(size))
      } else {
        Ok(vec![])
      };
    let mut nested_attribs: Vec<Attribute> = nested_attribs?;
    attributes.attribs.append(&mut nested_attribs);

    if object.text.len() > 0 {
      let text_attrib = Self::read_as_text(object)?;
      attributes.attribs.push(text_attrib);
      let p = attributes
        .position_mut()
        .expect("Text must have a position");
      p.0.y += object.height;
    }

    Ok(attributes)
  }


  /// Decompose the attributes into components and add them to the ECS.
  pub fn into_ecs<'a>(self, world: &mut World, z_level: ZLevel) -> Entity {
    let ent = world.create_entity().build();
    self.into_ecs_with_entity(ent, world, z_level);
    ent
  }

  pub fn into_ecs_with_entity<'a>(
    self,
    ent: Entity,
    world: &mut World,
    z_level: ZLevel,
  ) {
    let mut z_inc = 0;
    self.attribs.into_iter().for_each(|attrib| {
      match attrib {
        Attribute::Item(item) => {
          world
            .write_storage::<Item>()
            .insert(ent, item)
            .expect("Could not insert an Item component");
        }
        //Attribute::Script(script) => {
        //  world
        //    .write_storage::<Script>()
        //    .insert(ent, script)
        //    .expect("Could not insert Script component.");
        //}
        Attribute::Action(action) => {
          world
            .write_storage::<Action>()
            .insert(ent, action)
            .expect("Could not insert Action component.");
        }
        Attribute::Barrier(shape) => {
          world
            .write_storage::<Barrier>()
            .insert(ent, Barrier)
            .expect("Could not insert Barrier component.");
          world
            .write_storage::<Shape>()
            .insert(ent, shape)
            .expect("Could not insert Shape component.");
        }
        Attribute::Player(control) => {
          world
            .write_storage::<Player>()
            .insert(ent, control)
            .expect("Could not insert Player component.");
        }
        Attribute::Fence(f) => {
          world
            .write_storage::<Fence>()
            .insert(ent, f)
            .expect("Could not insert Fence component.");
        }
        Attribute::StepFence(f) => {
          world
            .write_storage::<StepFence>()
            .insert(ent, f)
            .expect("Could not insert StepFence component.");
        }
        //Lifespan(lifespan) => {
        //  world.insert(ent, lifespan);
        //}
        Attribute::MaxSpeed(max_speed) => {
          world
            .write_storage::<MaxSpeed>()
            .insert(ent, max_speed)
            .expect("Could not insert MaxSpeed component.");
        }
        Attribute::Name(name) => {
          world
            .write_storage::<Name>()
            .insert(ent, name)
            .expect("Could not insert Name component.");
        }
        Attribute::OriginOffset(origin_offset) => {
          world
            .write_storage::<OriginOffset>()
            .insert(ent, origin_offset)
            .expect("Could not insert OriginOffset component.");
        }
        Attribute::Position(position) => {
          world
            .write_storage::<Position>()
            .insert(ent, position)
            .expect("Could not insert Position component.");
        }
        Attribute::RenderingOrAnime(rendering_or_anime) => {
          rendering_or_anime.either(
            |r| {
              world
                .write_storage::<Rendering>()
                .insert(ent, r)
                .expect("Could not insert Rendering component.");
            },
            |l| {
              world
                .write_storage::<Animation>()
                .insert(ent, l)
                .expect("Could not insert Animation component.");
            },
          );
        }
        Attribute::Shape(s) => {
          world
            .write_storage::<Shape>()
            .insert(ent, s)
            .expect("Could not insert Shape component");
        }
        //Attribute::Sound(s) => {
        //  world
        //    .write_storage::<Sound>()
        //    .insert(ent, s)
        //    .expect("Could not insert Sound component");
        //}
        //Attribute::Music(m) => {
        //  world
        //    .write_storage::<Music>()
        //    .insert(ent, m)
        //    .expect("Could not insert Music component");
        //}
        Attribute::ZIncrement(z) => {
          z_inc = z;
        }
        Attribute::Inventory(n) => {
          let blank_inv = Inventory::new(vec![]);
          // Try to find the inventory
          let may_found = find_by::<Name, Inventory>(world, &Name(n));
          let inv = if let Some((_, inv)) = may_found {
            inv
          } else {
            // Otherwise return a blank one
            blank_inv
          };
          // And the inventory
          let mut inventories = world.write_storage::<Inventory>();
          inventories
            .insert(ent, inv)
            .expect("Could not insert Inventory component.");
        }
        Attribute::Zone(shape) => {
          world
            .write_storage::<Shape>()
            .insert(ent, shape)
            .expect("could not insert Zone shape");
          world
            .write_storage::<Zone>()
            .insert(ent, Zone { inside: vec![] })
            .expect("Could not insert Zone component.");
        }
      }
    });

    world
      .write_storage::<ZLevel>()
      .insert(ent, ZLevel(z_inc as f32 + z_level.0))
      .expect("Could not insert ZLevel component.");
  }


  /// ## Convenience functions for returning a specific attribute

  pub fn action(&self) -> Option<Action> {
    for a in &self.attribs {
      match a {
        Attribute::Action(p) => return Some(p.clone()),
        _ => {}
      }
    }
    None
  }

  pub fn barrier(&self) -> Option<Shape> {
    for a in &self.attribs {
      match a {
        Attribute::Barrier(p) => return Some(p.clone()),
        _ => {}
      }
    }
    None
  }

  pub fn control(&self) -> Option<Player> {
    for a in &self.attribs {
      match a {
        Attribute::Player(p) => return Some(p.clone()),
        _ => {}
      }
    }
    None
  }

  pub fn item(&self) -> Option<Item> {
    for a in &self.attribs {
      match a {
        Attribute::Item(p) => return Some(p.clone()),
        _ => {}
      }
    }
    None
  }

  pub fn name(&self) -> Option<Name> {
    for a in &self.attribs {
      match a {
        Attribute::Name(p) => return Some(p.clone()),
        _ => {}
      }
    }
    None
  }

  pub fn position(&self) -> Option<Position> {
    for a in &self.attribs {
      match a {
        Attribute::Position(p) => return Some(p.clone()),
        _ => {}
      }
    }
    None
  }

  pub fn position_mut(&mut self) -> Option<&mut Position> {
    for a in &mut self.attribs {
      match a {
        Attribute::Position(p) => return Some(p),
        _ => {}
      }
    }
    None
  }

  pub fn rendering_or_anime(&self) -> Option<Either<Rendering, Animation>> {
    for a in &self.attribs {
      match a {
        Attribute::RenderingOrAnime(r) => return Some(r.clone()),
        _ => {}
      }
    }
    None
  }

  pub fn rendering(&self) -> Option<Rendering> {
    for a in &self.attribs {
      match a {
        Attribute::RenderingOrAnime(r) => {
          match &r {
            Either::Left(rend) => {
              return Some(rend.clone());
            }

            _ => return None,
          };
        }
        _ => {}
      }
    }
    None
  }

  pub fn z_inc(&self) -> Option<i32> {
    for a in &self.attribs {
      match a {
        Attribute::ZIncrement(z) => return Some(*z),
        _ => {}
      }
    }
    None
  }

  pub fn max_speed(&self) -> Option<MaxSpeed> {
    for a in &self.attribs {
      match a {
        Attribute::MaxSpeed(m) => return Some(m.clone()),
        _ => {}
      }
    }
    None
  }

  pub fn origin_offset(&self) -> Option<OriginOffset> {
    for a in &self.attribs {
      match a {
        Attribute::OriginOffset(m) => return Some(m.clone()),
        _ => {}
      }
    }
    None
  }

  //pub fn script(&self) -> Option<Script> {
  //  for a in &self.attribs {
  //    match a {
  //      Attribute::Script(m) => return Some(m.clone()),
  //      _ => {}
  //    }
  //  }
  //  None
  //}

  pub fn shape(&self) -> Option<Shape> {
    for a in &self.attribs {
      match a {
        Attribute::Shape(m) => return Some(m.clone()),
        _ => {}
      }
    }
    None
  }

  pub fn shape_mut(&mut self) -> Option<&mut Shape> {
    for a in &mut self.attribs {
      match a {
        Attribute::Shape(m) => return Some(m),
        _ => {}
      }
    }
    None
  }

  // TODO: Add the web sound module
  //pub fn sound(&self) -> Option<Sound> {
  //  for a in &self.attribs {
  //    match a {
  //      Attribute::Sound(m) => { return Some(m.clone()) }
  //      _ => {}
  //    }
  //  }
  //  None
  //}


  /// Provide a best guess as to the scale of the entity.
  /// This is used to scale child objects such as origins and barriers.
  pub fn scale(&self) -> V2 {
    self
      .rendering_or_anime()
      .map(|e| {
        e.either(
          |rendering| {
            rendering
              .as_frame()
              .map(|frame| frame.scale())
              .unwrap_or(V2::new(1.0, 1.0))
          },
          |anime| {
            anime
              .frames
              .get(0)
              .map(|anime_frame| {
                anime_frame
                  .rendering
                  .as_frame()
                  .map(|frame| frame.scale())
                  .unwrap_or(V2::new(1.0, 1.0))
              })
              .unwrap_or(V2::new(1.0, 1.0))
          },
        )
      })
      .unwrap_or(V2::new(1.0, 1.0))
  }
}
