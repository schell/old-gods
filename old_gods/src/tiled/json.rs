//! Tiled map editor types and operations.
//!
//! TODO: Investigate whether we can support external tilesets on web
use log::trace;
use serde::de::{Deserialize, Deserializer};
use serde_json::{from_reader, from_str, Error, Value};
use specs::prelude::{Component as SpecsComponent, HashMapStorage};
use std::collections::HashMap;
use std::fs::File;
use std::future::Future;
use std::io::BufReader;
use std::path::{Component, Path, PathBuf};
use std::result::Result;
use std::vec::Vec;

use super::super::geom::V2;


/// #Tile-flipping constants
/// See https://docs.mapeditor.org/en/latest/reference/tmx-map-format/#tile-flipping
const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x80000000;
const FLIPPED_VERTICALLY_FLAG: u32 = 0x40000000;
const FLIPPED_DIAGONALLY_FLAG: u32 = 0x20000000;


#[derive(Deserialize, Clone, Hash, PartialEq, Eq, Debug)]
pub struct GlobalId(pub u32);

impl GlobalId {
  pub fn convert_to_local(&self, gid: &GlobalId) -> LocalId {
    let GlobalId(fgid) = self;
    let GlobalId(tgid) = gid;
    LocalId(tgid - fgid)
  }
}


#[derive(Clone, Hash, PartialEq, Debug)]
pub struct GlobalTileIndex {
  pub id: GlobalId,
  pub is_flipped_horizontally: bool,
  pub is_flipped_vertically: bool,
  pub is_flipped_diagonally: bool,
}


impl<'de> Deserialize<'de> for GlobalTileIndex {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let bits: u32 = u32::deserialize(deserializer)?;
    let is_flipped_horizontally = (bits & FLIPPED_HORIZONTALLY_FLAG) > 0;
    let is_flipped_vertically = (bits & FLIPPED_VERTICALLY_FLAG) > 0;
    let is_flipped_diagonally = (bits & FLIPPED_DIAGONALLY_FLAG) > 0;
    let id = GlobalId(
      bits
        & !(FLIPPED_HORIZONTALLY_FLAG
          | FLIPPED_VERTICALLY_FLAG
          | FLIPPED_DIAGONALLY_FLAG),
    );
    Ok(GlobalTileIndex {
      id,
      is_flipped_diagonally,
      is_flipped_vertically,
      is_flipped_horizontally,
    })
  }
}


#[derive(Deserialize, Clone, Hash, PartialEq, Eq, Debug)]
pub struct LocalId(pub u32);

fn no() -> bool {
  false
}

#[derive(Deserialize, Clone, Debug)]
pub struct Point<T> {
  pub x: T,
  pub y: T,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum TextValue {
  String(String),
  Int(u16),
  Bool(bool),
}


impl TextValue {
  pub fn get_string(&self) -> Option<String> {
    match self {
      TextValue::String(s) => Some(s.clone()),
      _ => None,
    }
  }

  pub fn get_uint(&self) -> Option<u16> {
    match self {
      TextValue::Int(i) => Some(*i),
      _ => None,
    }
  }

  pub fn get_bool(&self) -> Option<bool> {
    match self {
      TextValue::Bool(b) => Some(*b),
      _ => None,
    }
  }
}


#[derive(Deserialize, Clone, Debug)]
pub struct Object {
  pub id: u32,

  pub width: f32,

  pub height: f32,

  pub name: String,

  #[serde(rename = "type")]
  pub type_is: String,

  #[serde(default)]
  pub properties: Vec<Property>,

  pub visible: bool,

  pub x: f32,

  pub y: f32,

  pub rotation: f32,

  pub gid: Option<GlobalTileIndex>,

  #[serde(default = "no")]
  pub ellipse: bool,

  #[serde(default = "no")]
  pub point: bool,

  pub polygon: Option<Vec<Point<f32>>>,

  pub polyline: Option<Vec<Point<f32>>>,

  #[serde(default)]
  pub text: HashMap<String, TextValue>,
}


impl SpecsComponent for Object {
  type Storage = HashMapStorage<Self>;
}


impl Object {
  pub fn get_all_properties(&self, map: &Tiledmap) -> Vec<Property> {
    let mut object_properties = self.properties.clone();
    let mut tile_properties = if let Some(gid) = &self.gid {
      if let Some(tile) = map.get_tile(&gid.id) {
        tile.properties.clone()
      } else {
        vec![]
      }
    } else {
      vec![]
    };
    object_properties.append(&mut tile_properties);
    object_properties
  }


  pub fn json_properties(&self) -> HashMap<String, Value> {
    self
      .properties
      .iter()
      .map(|p| (p.name.clone(), p.value.clone()))
      .collect()
  }


  /// Return the type of the object. If the type is not specified
  /// at the object level, descend into any underlying tile data
  /// to find the type.
  pub fn get_deep_type(&self, map: &Tiledmap) -> String {
    if self.type_is.is_empty() {
      if let Some(ref gid) = self.gid {
        if let Some(tile) = map.get_tile(&gid.id) {
          return tile.type_is.clone();
        }
      }
    }
    self.type_is.clone()
  }
}


fn topdown() -> String {
  "topdown".to_string()
}
fn empty() -> String {
  "".to_string()
}


#[derive(Deserialize, Clone, Debug)]
pub struct TileLayerData {
  /// Column count. Same as map width for fixed-size maps.
  pub width: u32,

  /// Row count. Same as map height for fixed-size maps.
  pub height: u32,

  /// Array of tile indices.
  pub data: Vec<GlobalTileIndex>,
}


#[derive(Deserialize, Clone, Debug)]
pub struct ObjectLayerData {
  /// “topdown” (default) or “index”. objectgroup only.
  #[serde(default = "topdown")]
  pub draworder: String,

  /// Array of Objects. objectgroup only.
  pub objects: Vec<Object>,
}


#[derive(Deserialize, Clone, Debug)]
pub struct LayerLayerData {
  pub layers: Vec<Layer>,
}


#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum LayerData {
  Tiles(TileLayerData),
  Objects(ObjectLayerData),
  Layers(LayerLayerData),
}


#[derive(Deserialize, Clone, Debug)]
pub struct Layer {
  /// Name assigned to this layer
  pub name: String,

  /// “tilelayer”, “objectgroup”, or “imagelayer”
  #[serde(rename = "type")]
  pub type_is: String,

  /// Whether layer is shown or hidden in editor
  pub visible: bool,

  /// Horizontal layer offset in tiles. Always 0.
  pub x: i32,

  /// Vertical layer offset in tiles. Always 0.
  pub y: i32,

  /// string key-value pairs.
  #[serde(default)]
  pub properties: HashMap<String, String>,

  /// Value between 0 and 1
  pub opacity: f32,

  /// The layer's data which depends on the type of layer.
  #[serde(flatten)]
  pub layer_data: LayerData,
}


impl Layer {
  pub fn get_z(&self) -> Option<i32> {
    let s = self.properties.get("z")?;
    from_str(s).expect("Could not read layer z")
  }
  pub fn get_z_inc(&self) -> Option<i32> {
    let s = self.properties.get("z_inc")?;
    from_str(s).expect("Could not read layer z_inc")
  }

  pub fn is_group(&self) -> bool {
    match self.layer_data {
      LayerData::Layers(_) => true,
      _ => false,
    }
  }

  /// Return this layer's objects, or an empty vector.
  pub fn objects(&self) -> Vec<&Object> {
    match &self.layer_data {
      LayerData::Objects(objects) => objects.objects.iter().collect(),
      _ => vec![],
    }
  }


  /// Return the first object with the given name, if possible
  pub fn get_object_by_name(&self, name: &str) -> Option<&Object> {
    for obj in self.objects() {
      if obj.name == name {
        return Some(obj);
      }
    }
    None
  }
}


#[derive(Deserialize, Clone, Debug)]
pub struct Terrain {
  /// Name of terrain
  pub name: String,
  /// Local ID of tile representing terrain
  pub tile: LocalId,
}


#[derive(Deserialize, Clone, Debug)]
pub struct Frame {
  pub duration: u32,
  pub tileid: LocalId,
}


#[derive(Deserialize, Clone, Debug)]
pub struct ObjectGroup {
  pub draworder: String,

  pub name: String,

  pub objects: Vec<Object>,

  pub opacity: f32,

  #[serde(rename = "type")]
  pub type_is: String,

  pub visible: bool,

  pub x: u32,

  pub y: u32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Property {
  pub name: String,

  #[serde(rename = "type")]
  pub type_is: String,

  pub value: Value,
}


#[derive(Deserialize, Clone, Debug)]
pub struct Tile {
  pub id: LocalId,

  #[serde(rename = "type")]
  #[serde(default = "empty")]
  pub type_is: String,

  #[serde(default)]
  pub properties: Vec<Property>,

  #[serde(rename = "objectgroup")]
  pub object_group: Option<ObjectGroup>,

  pub animation: Option<Vec<Frame>>,
}

impl Tile {
  /// The object with the given name, if possible.
  pub fn _object_with_name(&self, name: &String) -> Option<&Object> {
    let group = self.object_group.as_ref()?;
    for obj in &group.objects {
      if obj.name == *name {
        return Some(&obj);
      }
    }
    None
  }

  /// The object with the given type, if possible.
  pub fn object_with_type(&self, type_is: &String) -> Option<&Object> {
    let group = self.object_group.as_ref()?;
    for obj in &group.objects {
      if obj.type_is == *type_is {
        return Some(&obj);
      }
    }
    None
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tileset {
  /// Image used for tiles in this set
  pub image: String,

  /// Name given to this tileset
  pub name: String,

  /// Maximum width of tiles in this set
  pub tilewidth: u32,

  /// Maximum height of tiles in this set
  pub tileheight: u32,

  /// Width of source image in pixels
  pub imagewidth: u32,

  /// Height of source image in pixels
  pub imageheight: u32,

  /// String key-value pairs
  #[serde(default)]
  pub properties: HashMap<String, String>,

  /// String key-value pairs
  #[serde(default)]
  pub propertytypes: HashMap<String, String>,

  /// Buffer between image edge and first tile (pixels)
  pub margin: u32,

  /// Spacing between adjacent tiles in image (pixels)
  pub spacing: u32,

  /// Per-tile properties
  #[serde(default)]
  pub tileproperties: HashMap<LocalId, Vec<Property>>,

  /// Array of Terrains (optional)
  #[serde(default)]
  pub terrains: Vec<Terrain>,

  /// The number of tile columns in the tileset
  pub columns: u32,

  /// The number of tiles in this tileset
  pub tilecount: u32,

  /// Tiles (optional)
  #[serde(default)]
  pub tiles: Vec<Tile>,
}


#[derive(Debug, Clone, PartialEq, Hash)]
pub struct AABB<T> {
  pub x: T,
  pub y: T,
  pub w: T,
  pub h: T,
}


impl Tileset {
  /// Given a GlobalId, return the rectangle (x, y, w, h) of the tile at that
  /// index in this Tileset, if it is indeed contained within this Tileset.
  pub fn aabb_of_tile_index(&self, ndx: u32) -> Option<AABB<u32>> {
    if ndx < self.tilecount {
      let tw = self.tilewidth;
      let th = self.tileheight;
      let m = self.margin;
      let s = self.spacing;
      let nc = self.columns;
      let yndx = ndx / nc;
      let xndx = ndx % nc;
      let x = m + (tw + s) * xndx;
      let y = m + (th + s) * yndx;
      Some(AABB {
        x: x,
        y: y,
        w: tw,
        h: th,
      })
    } else {
      None
    }
  }


  /// Return the source AABB of the given tile's LocalId in this Tileset,
  /// if possible.
  pub fn aabb_local(&self, lid: &LocalId) -> Option<AABB<u32>> {
    let LocalId(local_ndx) = lid;
    if *local_ndx < self.tilecount {
      self.aabb_of_tile_index(*local_ndx)
    } else {
      None
    }
  }


  /// Return the source AABB of the given tile GlobalId in this Tileset,
  /// if possible.
  pub fn aabb(
    &self,
    firstgid: &GlobalId,
    tilegid: &GlobalId,
  ) -> Option<AABB<u32>> {
    let lid = firstgid.convert_to_local(tilegid);
    self.aabb_local(&lid)
  }


  /// Return the Tile with the given gid in this Tileet, if possible.
  pub fn tile(&self, firstgid: &GlobalId, tilegid: &GlobalId) -> Option<&Tile> {
    let l = firstgid.convert_to_local(tilegid);
    for tile in &self.tiles {
      if tile.id == l {
        return Some(tile);
      }
    }
    None
  }


  pub fn extend_tiles_with_tileproperties(&mut self) {
    for tile in self.tiles.iter_mut() {
      let mut props = self
        .tileproperties
        .get(&tile.id)
        .map(|ps| ps.clone())
        .unwrap_or(vec![]);
      tile.properties.append(&mut props);
    }
  }
}


/// An externally defined tileset.
#[derive(Deserialize, Debug, Clone)]
// #[serde(transparent)]
pub struct TilesetSource {
  /// A path to a tileset file, relative to its owner.
  pub source: String,
}


#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum TilesetPayload {
  Embedded(Tileset),
  Source(TilesetSource),
}


/// When a Tileset lives in a Tiledmap it may be an embedded
/// Tileset or a link to an external file, but both live inline with a
/// firstgid property.
#[derive(Deserialize, Clone, Debug)]
pub struct TilesetItem {
  /// GID corresponding to the first tile in the set
  pub firstgid: GlobalId,
  /// The item.
  #[serde(flatten)]
  pub payload: TilesetPayload,
}


impl TilesetItem {
  /// Hydrate any TilesetItem into a Tileset.
  /// This loads any external tilesets, as well as extends each Tile's properties
  /// to include thos in the set's tileproperties.
  pub fn hydrate_tileset(&self, path_prefix: &Path) -> Result<Tileset, Error> {
    match &self.payload {
      TilesetPayload::Embedded(s) => {
        let mut set = s.clone();
        set.extend_tiles_with_tileproperties();
        Ok(set)
      }
      TilesetPayload::Source(s) => {
        let path: PathBuf = path_prefix.join(Path::new(&s.source));
        trace!("Hydrating tileset with path {:?}", path);
        let file = File::open(path.clone()).unwrap();
        let reader = BufReader::new(file);
        from_reader(reader).map(|s: Tileset| {
          let img_path: PathBuf =
            path.parent().unwrap().to_path_buf().join(s.image.clone());
          let mut s = s;
          s.image = img_path.to_str().unwrap().to_string();
          s.extend_tiles_with_tileproperties();
          s
        })
      }
    }
  }

  /// Return the Tileset, if possible.
  pub fn tileset(&self) -> Option<&Tileset> {
    match &self.payload {
      TilesetPayload::Embedded(s) => Some(s),
      _ => None,
    }
  }
}


/// Our top level tiled map.
#[derive(Deserialize, Clone, Debug)]
pub struct Tiledmap {
  /// The JSON format version
  pub version: f32,

  /// The Tiled version used to save the file
  pub tiledversion: String,

  /// Number of tile columns
  pub width: i32,

  /// Number of tile rows
  pub height: i32,

  /// Map grid width.
  pub tilewidth: i32,

  /// Map grid height.
  pub tileheight: i32,

  /// Orthogonal, isometric, or staggered
  pub orientation: String,

  /// Array of Layers
  pub layers: Vec<Layer>,

  /// Array of Tilesets
  pub tilesets: Vec<TilesetItem>,

  /// Hex-formatted color (#RRGGBB or #AARRGGBB) (optional)
  pub backgroundcolor: Option<String>,

  /// Rendering direction (orthogonal maps only)
  pub renderorder: String,

  /// String key-value pairs
  #[serde(default)]
  pub properties: Vec<Property>,

  /// Auto-increments for each placed object
  pub nextobjectid: i32,
}

// TODO: Keep Tiledmap from panicking on parse failure.
impl Tiledmap {
  pub fn from_text(text: &str) -> Result<Tiledmap, String> {
    from_str(text).map_err(|e| format!("{}", e))
  }

  pub fn from_file(file: &str) -> Tiledmap {
    Self::new(Path::new(file))
  }

  pub async fn from_url<F, R>(
    base_url: &str,
    path: &str,
    load: F,
  ) -> Result<Tiledmap, String>
  where
    F: Fn(&str) -> R,
    R: Future<Output = Result<String, String>>,
  {
    let url = format!("{}/{}", base_url, path);
    let data = load(&url).await?;
    let mut tiledmap: Tiledmap =
      from_str(&data).map_err(|e| format!("{}", e))?;
    tiledmap
      .hydrate_tilesets_async(base_url, path, load)
      .await?;
    Ok(tiledmap)
  }

  pub fn new(path: &Path) -> Tiledmap {
    trace!("Opening Tiled map file {:?}", path);
    let file = File::open(path)
      .expect(&format!("Could not open the file '{:?}'.", path));
    let reader = BufReader::new(file);
    let m1: Tiledmap = from_reader(reader).expect("Could not read a file.");
    if let Some(parent) = path.parent() {
      m1.hydrate_tilesets(parent).expect(&format!(
        "Could not hydrate a Tileset in directory '{:?}'.",
        parent
      ))
    } else {
      m1
    }
  }

  /// Hydrate all tilesets, async.
  pub async fn hydrate_tilesets_async<F, R>(
    &mut self,
    base_url: &str,
    map_url: &str,
    load: F,
  ) -> Result<(), String>
  where
    F: Fn(&str) -> R,
    R: Future<Output = Result<String, String>>,
  {
    for mut item in self.tilesets.iter_mut() {
      // TODO: Load tilesets in parallel
      match &mut item.payload {
        TilesetPayload::Embedded(_) => {
          if cfg!(arch = "wasm32") {
            // This is because Tiled has no way of knowing what the prefix to
            // assets could be and we need to intercept
            panic!("TODO: Embedded tilesets are not supported on wasm32/web");
          }
        }

        TilesetPayload::Source(src) => {
          let map_path = Path::new(map_url);
          let map_dir = map_path.parent().ok_or("map is not in a directory")?;
          let tileset_url = map_dir.join(&src.source);
          let full_tileset_url = Path::new(base_url).join(tileset_url.clone());
          let url_str = full_tileset_url
            .to_str()
            .expect("could not get Tileset url as &str");
          trace!(
            "hydrating tileset item async:\n  base_url: {}\n  map_url: {}\n  tileset src: {:?}\n  tileset_url: {}\n  full_tileset_url: {}",
            base_url,
            map_url,
            src,
            tileset_url.display(),
            url_str
          );
          let data = load(url_str).await?;
          trace!("  got Tileset data for url: {}", url_str);

          // Update the image location
          let mut tileset: Tileset = from_str(&data)
            .map_err(|e| format!("error reading Tileset {}: {}", url_str, e))?;
          let tileset_dir =
            tileset_url.parent().expect("Tileset has no parent");
          let mut image_path = PathBuf::new();
          let image_path_joined = tileset_dir.join(&tileset.image);
          let image_path_components = image_path_joined.components();
          for next in image_path_components {
            let is_parent = next == Component::ParentDir;
            trace!("  component {:?} is parent {}", next, is_parent);
            if is_parent {
              image_path.pop();
            } else {
              image_path.push(next);
            }
          }
          trace!(
            "  converted url\n       {} \n  into {}",
            image_path_joined.display(),
            image_path.display()
          );
          let mut final_image_url = PathBuf::new();
          final_image_url.push(base_url);
          final_image_url.push(image_path);
          tileset.image = final_image_url
            .to_str()
            .expect("could not get canonical tileset url")
            .into();
          trace!("  final url is {}", tileset.image);
          tileset.extend_tiles_with_tileproperties();
          item.payload = TilesetPayload::Embedded(tileset);
        }
      }
    }
    Ok(())
  }


  /// Hydrate all tilesets and return them in a map.
  pub fn hydrate_tilesets(self, path_prefix: &Path) -> Result<Tiledmap, Error> {
    let mut tm = self.clone();
    for mut item in tm.tilesets.iter_mut() {
      let tileset = item.hydrate_tileset(path_prefix)?;
      item.payload = TilesetPayload::Embedded(tileset);
    }
    Ok(tm)
  }

  /// Return the Tileset that contains the GlobalId. If the Tilemap has not
  /// hydrated its tilesets, this will always return None.
  /// @see hydrate_tilesets
  pub fn get_tileset_by_gid(
    &self,
    global_id: &GlobalId,
  ) -> Option<(&GlobalId, &Tileset)> {
    match &global_id {
      GlobalId(0) => None,
      GlobalId(gid) => {
        for item in self.tilesets.iter() {
          let GlobalId(fgid) = item.firstgid;
          let set = item.tileset().expect("could not get tileset");
          if *gid >= fgid && *gid < (fgid + set.tilecount) {
            return Some((&item.firstgid, set));
          }
        }
        None
      }
    }
  }

  pub fn get_tile(&self, gid: &GlobalId) -> Option<&Tile> {
    let (firstgid, tileset) = self.get_tileset_by_gid(gid)?;
    tileset.tile(firstgid, gid)
  }

  pub fn get_tile_object_group(
    &self,
    tile_gid: &GlobalId,
  ) -> Option<ObjectGroup> {
    let tile = self.get_tile(tile_gid)?;
    tile.object_group.clone()
  }


  /// Return the layer with the given name
  pub fn get_layer_with_name(&self, name: &str) -> Option<&Layer> {
    for layer in &self.layers {
      if layer.name == name.to_string() {
        return Some(&layer);
      }
    }
    None
  }

  /// The sprite offset is an offset applied to sprites that are defined in an
  /// external map file. It is used to determine the offset to apply to the
  /// objects within the map file in order for them to line up correctly on
  /// the map.
  ///
  /// For this to work, the map itself must have the 'sprite_offset' property
  /// defined.
  pub fn get_sprite_offset(&self) -> Option<V2> {
    let value = self.get_property_by_name("sprite_offset")?;
    let offset_value: &str = value.as_str()?;
    let p: V2 =
      from_str(offset_value).expect("Could not deserialize sprite offset.");
    Some(p.scalar_mul(-1.0))
  }

  /// Get a custom proprety by name.
  pub fn get_property_by_name(&self, name: &str) -> Option<&Value> {
    for prop in &self.properties {
      if prop.name == name {
        return Some(&prop.value);
      }
    }
    None
  }

  /// Get the suggested size of the viewport.
  /// First looks to custom properties on the map called `viewport_width_tiles`
  /// and `viewport_height_tiles` or falls back to `viewport_width` and
  /// `viewport_height`.
  pub fn get_suggested_viewport_size(&self) -> Option<(u32, u32)> {
    let tile_width = self.tilewidth as u32;
    let tile_height = self.tileheight as u32;
    let width = self
      .get_property_by_name("viewport_width_tiles")
      .map(|num_tiles_value: &Value| {
        num_tiles_value
          .as_u64()
          .map(|num_tiles| num_tiles as u32 * tile_width)
      })
      .flatten()
      .or(
        self
          .get_property_by_name("viewport_width")
          .map(|value| value.as_u64().map(|width| width as u32))
          .flatten(),
      )?;
    let height = self
      .get_property_by_name("viewport_height_tiles")
      .map(|num_tiles_value: &Value| {
        num_tiles_value
          .as_u64()
          .map(|num_tiles| num_tiles as u32 * tile_height)
      })
      .flatten()
      .or(
        self
          .get_property_by_name("viewport_height")
          .map(|value| value.as_u64().map(|height| height as u32))
          .flatten(),
      )?;
    Some((width, height))
  }
}


impl SpecsComponent for Tiledmap {
  type Storage = HashMapStorage<Self>;
}
