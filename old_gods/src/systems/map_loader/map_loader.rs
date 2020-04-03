use serde_json::Value;
use specs::prelude::*;
use std::collections::HashMap;
use std::path::Path;

use super::super::super::prelude::{
  hex_color, Attribute, Attributes, BackgroundColor, CanBeEmpty, Color,
  GlobalTileIndex, InventoryLayer, ItemRecord, Layer, LayerData, Object,
  Position, Screen, Sprite, Tiledmap, ToonRecord, ZLevel, V2,
};

/// The result of loading one or more Tiled layers into the ECS.
pub struct LoadedLayers {
  /// All the top-level entities loaded
  pub top_level_entities: Vec<Entity>,

  /// A HashMap of all entities loaded within a layer group, keyed by the group
  /// layer's name.
  pub groups: HashMap<String, Vec<Entity>>,
}


impl LoadedLayers {
  pub fn new() -> LoadedLayers {
    LoadedLayers {
      top_level_entities: vec![],
      groups: HashMap::new(),
    }
  }

  pub fn all_entities(&self) -> Vec<Entity> {
    let mut tops = self.top_level_entities.clone();
    let mut groups: Vec<Entity> =
      self.groups.values().flat_map(|es| es.clone()).collect();
    tops.append(&mut groups);
    tops
  }

  pub fn append_entities(&mut self, other: LoadedLayers) {
    let other_tops = other.top_level_entities.into_iter();
    let other_groups = other.groups.into_iter();
    self.top_level_entities.extend(other_tops);
    self.groups.extend(other_groups);
  }
}


pub struct MapLoader<'a> {
  loaded_maps: HashMap<String, Tiledmap>,
  pub z_level: ZLevel,
  pub world: &'a mut World,
  pub origin: V2,
  pub layer_group: Option<String>,
  pub sprite: Option<Entity>,
}


impl<'a> MapLoader<'a> {
  pub fn load_it(file: String, lazy: &LazyUpdate) {
    lazy.exec_mut(|world| {
      let mut loader = MapLoader::new(world);
      let file = file;
      let map: &Tiledmap = loader.load_map(&file);
      // Get the background color based on the loaded map
      let bg: Color = map
        .backgroundcolor
        .as_ref()
        .map(|s: &String| {
          hex_color(s.as_str())
            .map_err(|e| format!("{:?}", e))
            .map(|(_, c)| c)
        })
        .unwrap_or(Ok(Color::rgb(0, 0, 0)))
        .unwrap()
        .clone();
      let width: u32 = map
        .get_property_by_name("viewport_width")
        .map(|value: &Value| {
          value
            .as_i64()
            .expect("map's 'viewport_width' property type must be unsigned int")
            as u32
        })
        .unwrap_or(map.width as u32 * map.tilewidth as u32);

      // Get the screen size based on the loaded map
      let height: u32 = map
        .get_property_by_name("viewport_height")
        .map(|value: &Value| {
          value.as_i64().expect(
            "map's 'viewport_height' property type must be unsigned int",
          ) as u32
        })
        .unwrap_or(map.height as u32 * map.tileheight as u32);

      let res = loader.load(&file, None, None);
      match res {
        Ok(_) => {}
        Err(msg) => panic!(msg),
      }

      let mut screen = world.write_resource::<Screen>();
      screen.set_size((width, height));

      let mut background_color = world.write_resource::<BackgroundColor>();
      background_color.0 = bg;
    });
  }

  /// Create a new MapLoader
  pub fn new<'c>(world: &'c mut World) -> MapLoader<'c> {
    MapLoader {
      loaded_maps: HashMap::new(),
      z_level: ZLevel(0.0),
      world,
      origin: V2::new(0.0, 0.0),
      layer_group: None,
      sprite: None,
    }
  }

  fn load_map(&mut self, file: &String) -> &Tiledmap {
    if !self.loaded_maps.contains_key(file) {
      let map: Tiledmap = Tiledmap::new(&Path::new(&file.clone()));
      self.loaded_maps.insert(file.clone(), map.clone());
    }
    self.loaded_maps.get(file).expect("Impossible!")
  }


  /// Sort the layers of a Tiledmap (in place) so that the layers
  /// process correctly. Really we just want the inventories
  /// layer to be loaded first.
  pub fn sort_layers(&self, layers: &mut Vec<Layer>) {
    let mut mndx = None;
    'find_ndx: for (layer, i) in layers.iter().zip(0..) {
      if let LayerData::Objects(_) = layer.layer_data {
        if layer.name == "inventories" {
          mndx = Some(i);
          break 'find_ndx;
        }
      }
    }
    if let Some(ndx) = mndx {
      let inv_layer = layers.remove(ndx);
      layers.insert(0, inv_layer);
    }
  }


  pub fn insert_map(
    &mut self,
    map: &mut Tiledmap,
    layer_group: Option<String>,
    sprite: Option<Entity>,
  ) -> Result<LoadedLayers, String> {
    self.sort_layers(&mut map.layers);
    let prev_group = self.layer_group.take();
    self.layer_group = layer_group;
    self.sprite = sprite;

    let res = self.load_layers(&map.layers, &map)?;

    self.layer_group = prev_group;

    Ok(res)
  }


  /// Load an entire top-level map into the ECS.
  /// Takes the file to load and optionally a layer group to load. If a layer
  /// group is provided only layers within the group will be loaded. If no layer
  /// group is provided all layers will be loaded.
  /// Returns an error or a tuple consisting of
  pub fn load(
    &mut self,
    file: &String,
    layer_group: Option<String>,
    sprite: Option<Entity>,
  ) -> Result<LoadedLayers, String> {
    self.load_map(&file);

    let mut map = self
      .loaded_maps
      .get(file)
      .expect("Could not retreive map.")
      .clone();

    self.insert_map(&mut map, layer_group, sprite)
  }

  /// Possibly Increments the ZLevel based on layer properties
  fn increment_z_by_layer(&mut self, layer: &Layer) {
    let z_inc = layer.get_z_inc().unwrap_or(0);
    if z_inc != 0 {
      self.z_level.0 += z_inc as f32;
      println!(
        "incrementing ZLevel to {:?} - layer {:?}",
        self.z_level, layer.name
      );
    }
  }

  /// Load one layer of LayerData.
  fn load_layer_data(
    &mut self,
    layer_name: &String,
    data: &LayerData,
    map: &Tiledmap,
  ) -> Result<Vec<Entity>, String> {
    println!("load_layer_data: {} at z:{:?}", layer_name, self.z_level);

    match data {
      LayerData::Tiles(tiles) => Ok(self.load_tile_layer(&tiles.data, map)?),
      LayerData::Objects(objects) => {
        if layer_name == "inventories" {
          let inv_layer: InventoryLayer =
            InventoryLayer::read(map, &objects.objects)?;

          let top_level_entities =
            inv_layer.into_ecs(self.world, self.z_level)?;
          Ok(top_level_entities)
        } else {
          let top_level_entities = objects.objects.iter().fold(
            Ok(vec![]),
            |result: Result<Vec<Entity>, String>, obj: &Object| {
              let ent = self.load_top_level_object(obj, map)?;
              let mut ents = result?;
              ents.push(ent);
              Ok(ents)
            },
          )?;
          Ok(top_level_entities)
        }
      }
      LayerData::Layers(layers) => {
        layers.layers.iter().fold(Ok(vec![]), |res, layer| {
          let mut res = res?;
          self.increment_z_by_layer(&layer);
          let mut ents =
            self.load_layer_data(&layer.name, &layer.layer_data, map)?;
          res.append(&mut ents);
          Ok(res)
        })
      }
    }
  }

  /// Load a vec of layers into the ECS
  fn load_layers(
    &mut self,
    layers: &Vec<Layer>,
    map: &Tiledmap,
  ) -> Result<LoadedLayers, String> {
    let variant = self.layer_group.take();
    // First figure out which layers we need to load
    let layers_to_load: Vec<&Layer> = if variant.is_some() {
      let variant_name = variant.as_ref().unwrap();
      // Only get the variant layers
      layers
        .iter()
        .filter_map(|layer| {
          if layer.name == *variant_name {
            match &layer.layer_data {
              LayerData::Layers(variant_layers) => {
                let variant_layers: Vec<&Layer> =
                  variant_layers.layers.iter().collect();
                Some(variant_layers)
              }
              _ => None,
            }
          } else {
            None
          }
        })
        .flatten()
        .collect()
    } else {
      // Return the layers as normal
      layers.iter().collect()
    };

    let mut layers = LoadedLayers::new();
    for layer in layers_to_load.iter() {
      self.increment_z_by_layer(&layer);
      let mut ents =
        self.load_layer_data(&layer.name, &layer.layer_data, map)?;

      // If this layer is part of a group, add it as a keyframe
      if layer.is_group() {
        layers.groups.insert(layer.name.clone(), ents);
      } else {
        layers.top_level_entities.append(&mut ents);
      }
    }

    Ok(layers)
  }

  /// ## Loading tiles

  /// Load a vector of tiles keyed by their GlobalId.
  fn load_tile_layer(
    &mut self,
    tiles: &Vec<GlobalTileIndex>,
    map: &Tiledmap,
  ) -> Result<Vec<Entity>, String> {
    let (width, height) = (map.width as u32, map.height as u32);
    let tw = map.tilewidth as u32;
    let th = map.tileheight as u32;
    println!("  layer width {:?} and height {:?}", width, height);
    tiles
      .iter()
      .zip(0..)
      .fold(Ok(vec![]), |result, (gid, ndx)| {
        let mut ents = result?;
        let yndx = ndx / width;
        let xndx = ndx % width;
        println!("    tile {:?} ({:?}, {:?})", ndx, xndx, yndx);
        let tile_origin =
          self.origin + V2::new((tw * xndx) as f32, (th * yndx) as f32);
        let mut attribs = Attributes::read_gid(map, gid, None)?;
        attribs.push(Attribute::Position(Position(tile_origin)));
        let attributes = Attributes { attribs };

        let ent = attributes.into_ecs(self.world, self.z_level);
        ents.push(ent);
        Ok(ents)
      })
  }

  /// ## Loading objects

  /// Load a top level object.
  fn load_top_level_object(
    &mut self,
    object: &Object,
    map: &Tiledmap,
  ) -> Result<Entity, String> {
    let msg_name = object
      .name
      .non_empty()
      .map(|s| s.clone())
      .unwrap_or("unnamed".to_string());
    println!("Encountered top object '{}'", msg_name);

    match object.get_deep_type(map).as_str() {
      "character" => {
        // Load a character into the game world
        let mut rec = ToonRecord::read(map, object)?;
        rec.attributes.attribs = rec
          .attributes
          .attribs
          .into_iter()
          .map(|a| match a {
            Attribute::Position(p) => {
              Attribute::Position(Position(p.0 + self.origin))
            }
            a => a,
          })
          .collect();

        Ok(rec.into_ecs(self.world, self.z_level))
      }
      "item" => {
        // Load an item into the game world (not an inventory)
        let mut rec = ItemRecord::read(map, object)?;
        // Update the position to be offset by the origin passed in.
        rec
          .attributes
          .position_mut()
          .map(|pos| pos.0 += self.origin);

        Ok(rec.into_ecs(self.world, self.z_level))
      }
      "action" => {
        let mut attributes = Attributes::read(map, object)?;
        attributes.position_mut().map(|pos| pos.0 += self.origin);
        let _action = attributes.action().ok_or(format!(
          "Could not read action {:?}\nDid read:\n{:?}",
          object, attributes
        ))?;
        println!("Creating action:\n{:?}", attributes);
        Ok(attributes.into_ecs(self.world, self.z_level))
      }
      "sprite" => Sprite::read(self, map, object),

      "zone" | "fence" | "step_fence" => {
        let mut attributes = Attributes::read(map, object)?;
        attributes.position_mut().map(|p| {
          p.0 += self.origin + V2::new(0.0, object.height);
        });
        Ok(attributes.into_ecs(self.world, self.z_level))
      }

      "point" | "sound" | "music" => {
        let mut attributes = Attributes::read(map, object)?;
        attributes.position_mut().map(|p| {
          p.0 += self.origin;
        });
        Ok(attributes.into_ecs(self.world, self.z_level))
      }

      "barrier" => {
        let mut attributes = Attributes::read(map, object)?;
        let position = attributes
          .position_mut()
          .expect("Barrier object has no position");
        position.0 = self.origin;

        Ok(attributes.into_ecs(self.world, self.z_level))
      }

      ty => {
        let gid = object.gid.clone();
        if let Some(gid) = gid {
          // Tiled tiles' origin is at the bottom of the tile, not the top
          let y = object.y - object.height;
          let p = self.origin + V2::new(object.x, y);
          let size = (object.width as u32, object.height as u32);

          let mut attribs = Attributes::read_gid(map, &gid, Some(size))?;
          attribs.push(Attribute::Position(Position(p)));

          let props = object
            .properties
            .iter()
            .map(|p| (&p.name, p))
            .collect::<HashMap<_, _>>();
          let mut prop_attribs = Attributes::read_properties(&props)?;
          attribs.append(&mut prop_attribs);

          let attributes = Attributes { attribs };
          println!("  {:?} with attributes:{:?}", ty, attributes);

          Ok(attributes.into_ecs(self.world, self.z_level))
        } else {
          if object.text.len() > 0 {
            // This is a text object
            let mut attribs = Attributes::read(map, object)?;
            let p = attribs.position_mut().expect("Text must have a Position");
            p.0 += self.origin;
            println!(
              "  {:?} with attributes:{:?} and z_level:{:?}",
              ty, attribs, self.z_level
            );
            Ok(attribs.into_ecs(self.world, self.z_level))
          } else {
            Err(format!("Unsupported object\n{:?}", object))
          }
        }
      }
    }
  }
}
