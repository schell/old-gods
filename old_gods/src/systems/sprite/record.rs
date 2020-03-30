use serde_json::Value;
use specs::prelude::*;
use std::collections::HashMap;

use super::super::super::components::{
  Attribute, Attributes, Exile, Name, Script,
};
use super::super::super::systems::map_loader::{LoadedLayers, MapLoader};
use super::super::super::tiled::json::*;


/// Sprites are a collection of other entities.
/// Typically the entities will be defined within a custom Tiledmap.
/// See the OwnersManual for more info.
///
/// An entity with a Sprite component can be used to show/hide other entities.
/// This gives the effect of complex animation (animation of entities with
/// different components).
///
/// Sprites are defined using an entire Tiled map file.
#[derive(Debug, Clone, Default)]
pub struct Sprite {
  /// The keyframe tells the sprite what keyframe it should switch to.
  /// In order to change the keyframe of a sprite simply set this to
  /// `Some("desired keyframe")`.
  pub keyframe: Option<String>,

  /// The actual keyframe of this sprite.
  current_keyframe: String,

  /// The list of children at the top level of this sprite variant.
  pub top_level_children: Vec<Entity>,

  /// The keyframed children of this sprite, sorted by keyframe
  pub keyframe_children: HashMap<String, Vec<Entity>>,
}


impl Sprite {
  /// Creates a new sprite with only top level children
  pub fn with_top_level_children(top_level_children: Vec<Entity>) -> Sprite {
    Sprite {
      top_level_children,
      keyframe: None,
      current_keyframe: "".to_string(),
      keyframe_children: HashMap::new(),
    }
  }


  /// Parses a hashmap for the params needed to load a sprite from a Tiled map
  /// file.
  pub fn loading_params(
    hmap: HashMap<String, Value>,
  ) -> Result<(String, String, Option<String>), String> {
    let variant: &str = hmap
      .get("variant")
      .ok_or("Sprite is missing its 'variant' property")?
      .as_str()
      .ok_or("Sprite's variant property must be a string")?;
    let file: &str = hmap
      .get("file")
      .ok_or("Sprite is missing its 'file' property")?
      .as_str()
      .ok_or("Sprite's file proprety must be a string")?;
    let keyframe: Option<String> = hmap
      .get("keyframe")
      .map(|val: &Value| val.as_str().map(|s| s.to_string()))
      .flatten();
    Ok((variant.to_string(), file.to_string(), keyframe))
  }


  /// Construct the parameters to load a sprite.
  pub fn get_params<'a>(
    map: &'a Tiledmap,
    object: &'a Object,
  ) -> Result<(String, String, Option<String>), String> {
    let properties = object
      .get_all_properties(map)
      .into_iter()
      .map(|prop| (prop.name, prop.value))
      .collect::<HashMap<_, _>>();
    Sprite::loading_params(properties)
  }


  /// Switch the keyframe of this sprite. If a Sound storage is passed,
  /// play any sounds that may be set to auto_play=true.
  pub fn switch_keyframe(
    &mut self,
    keyframe: &String,
    exiles: &mut WriteStorage<Exile>,
  ) {
    self.keyframe = None;
    self.current_keyframe = keyframe.clone();

    for (child_keyframe, children) in &self.keyframe_children {
      for child in children {
        let is_exiled = child_keyframe != keyframe;
        if is_exiled {
          Exile::exile(*child, "sprite", exiles);
        } else {
          // Domesticate all the children in this keyframe
          Exile::domesticate(*child, "sprite", exiles);
        }
      }
    }
  }

  /// Return the children within the current keyframe
  pub fn current_children(&self) -> Vec<&Entity> {
    self
      .keyframe_children
      .get(&self.current_keyframe)
      .expect("A sprite does not contain children of its own keyframe")
      .into_iter()
      .collect()
  }

  /// Return the current keyframe
  pub fn current_keyframe(&self) -> &String {
    &self.current_keyframe
  }
}


impl Component for Sprite {
  type Storage = HashMapStorage<Sprite>;
}


impl Sprite {
  pub fn read(
    loader: &mut MapLoader,
    map: &Tiledmap,
    object: &Object,
  ) -> Result<Entity, String> {
    // Get the parameters to load the sprite
    let params = Sprite::get_params(map, object);
    println!("Loading sprite with params {:?}", params);
    // Load the sprite file
    let (variant, file, may_keyframe) =
      params.map_err(|e| format!("{}:\n{:?}", e, object))?;
    // Store the previous z
    let prev_z = loader.z_level;
    // Create an entity to hold our entities
    let mut attributes = Attributes::read(map, object)?;
    attributes.attribs = attributes
      .attribs
      .into_iter()
      .filter(|att| match att {
        Attribute::RenderingOrAnime(_) => false,
        Attribute::Barrier(_) => false,
        _ => true,
      })
      .collect();
    let position =
      attributes.position().ok_or("Sprite must have a position")?;
    let ent = loader.world.create_entity().build();
    let prev_origin = loader.origin;
    loader.origin = loader.origin + position.0;
    // Load the sprite's layers
    let layers: LoadedLayers =
      loader.load(&file, Some(variant.clone()), Some(ent))?;
    // Reset the loader's values
    loader.z_level = prev_z;
    loader.origin = prev_origin;
    let first_keyframe: Option<String> = layers
      .groups
      .keys()
      .collect::<Vec<_>>()
      .first()
      .map(|s: &&String| (*s).clone());
    let keyframe: Option<String> =
      may_keyframe.map(|s| s.clone()).or(first_keyframe);
    let current_keyframe = keyframe.unwrap_or("".to_string());
    let mut sprite = Sprite {
      keyframe: None,
      current_keyframe: current_keyframe.clone(),
      top_level_children: layers.top_level_entities,
      keyframe_children: layers.groups,
    };
    // Switch to the correct keyframe without playing sounds.
    sprite.switch_keyframe(
      &current_keyframe,
      &mut loader.world.write_storage::<Exile>(),
    );
    // Add the sprite component to the ent
    let _ = loader
      .world
      .write_storage::<Sprite>()
      .insert(ent, sprite.clone())
      .map_err(|e| format!("{:?}", e))?;

    // If it has a name component, do that too
    if let Some(name) = attributes.name() {
      loader
        .world
        .write_storage::<Name>()
        .insert(ent, name)
        .map_err(|e| format!("{:?}", e))?;
    }

    // If it has a script component, do that too
    println!("Does objcet have script? {:?}", object.properties);
    if let Some(script) = attributes.script() {
      println!("  with script {:?}", script);
      loader
        .world
        .write_storage::<Script>()
        .insert(ent, script)
        .map_err(|e| format!("{:?}", e))?;
    }

    // Print it all out for debugging
    let names = loader.world.read_storage::<Name>();
    let get_name = |ent| names.get(ent).map(|n| n.0.clone());

    println!("Created a sprite {:?}", get_name(ent));

    println!("  top_level_children:");
    let mut unnamed = 0;
    for ent in sprite.top_level_children {
      if let Some(name) = get_name(ent) {
        println!("    {:?}", name);
      } else {
        unnamed += 1;
      }
    }
    if unnamed > 0 {
      println!("    ...and {:?} unnamed children", unnamed);
    }

    if sprite.keyframe_children.keys().len() > 0 {
      println!("  keyframe_children:");
    }
    for (keyframe, children) in sprite.keyframe_children {
      println!("    keyframe: {}", keyframe);
      let mut unnamed = 0;
      for ent in children {
        if let Some(name) = get_name(ent) {
          println!("      {:?}", name);
        } else {
          unnamed += 1;
        }
      }
      if unnamed > 0 {
        println!("      ...and {:?} unnamed children", unnamed);
      }
    }

    Ok(ent)
    //println!(
    //  "Loading sprite map file '{}', variant {}, keyframe {:?} at {:?}",
    //  file,
    //  variant,
    //  keyframe,
    //  origin
    //);

    //let entity = self.load_sprite(
    //  &file.expect("Sprite is missing its definition file."),
    //  &variant.expect("Sprite is missing a variant."),
    //  keyframe,
    //  object.properties.get(&"inventory_name".to_string()),
    //  &(*object_origin + V2::new(object.x, object.y)),
    //);

    //let extra_comps = self.load_base_object(object, object_origin);
    //extra_comps
    //  .iter()
    //  .for_each(|c| self.add_component(entity, c));
    //return None
  }
}
