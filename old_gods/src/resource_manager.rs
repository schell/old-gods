//use sdl2::image::LoadTexture;
//use sdl2::video::WindowContext;
//use sdl2::render::{TextureCreator, Texture};
//use sdl2::ttf::{Font, Sdl2TtfContext};

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;


/// Generic trait to Load any Resource Kind
pub trait ResourceLoader<'l, R> {
  type Args: ?Sized;
  fn load(&'l self, data: &Self::Args) -> Result<R, String>;
}


/// TextureCreator knows how to load Textures
impl<'l, T> ResourceLoader<'l, Texture<'l>> for TextureCreator<T> {
  type Args = str;
  fn load(&'l self, path: &str) -> Result<Texture, String> {
    println!("Loading a texture: {:?}", path);
    self.load_texture(path)
  }
}


/// Font Context knows how to load Fonts
impl<'l> ResourceLoader<'l, Font<'l, 'static>> for Sdl2TtfContext {
  type Args = FontDetails;
  fn load(&'l self, details: &FontDetails) -> Result<Font<'l, 'static>, String> {
    println!("Loading a font: {:?}", details);
    self.load_font(&details.path, details.size)
  }
}


/// Generic struct to cache any resource loaded by a ResourceLoader
pub struct ResourceManager<'l, K, R, L>
where
  K: Hash + Eq,
  L: 'l + ResourceLoader<'l, R>
{
  loader: &'l L,
  cache: HashMap<K, R>,
}

impl<'l, K, R, L> ResourceManager<'l, K, R, L>
where
  K: Hash + Eq,
  L: ResourceLoader<'l, R>
{
  pub fn new(loader: &'l L) -> Self {
    ResourceManager {
      cache: HashMap::new(),
      loader: loader,
    }
  }

  pub fn get_cache(&self) -> &HashMap<K, R> {
    &self.cache
  }

  /// Generics magic to allow a HashMap to use String as a key
  /// while allowing it to use &str for gets
  pub fn load<D>(&mut self, details: &D) -> Result<&R, String>
  where
    L: ResourceLoader<'l, R, Args = D>,
    D: Debug + Eq + Hash + ?Sized,
    K: Borrow<D> + for<'a> From<&'a D>
  {
    let key =
      details
      .into();
    if !self.cache.contains_key(key) {
      let resource =
        self
        .loader
        .load(details)?;
      self
        .cache
        .insert(details.into(), resource);
    }
    self
      .cache
      .get(key)
      .ok_or(format!("Could not find resource {:?}", details))
  }

  /// Take a texture from the manager.
  pub fn take_resource<D>(&mut self, details: &D) -> Result<R, String>
  where
    L: ResourceLoader<'l, R, Args = D>,
    D: Debug + Eq + Hash + ?Sized,
    K: Borrow<D> + for<'a> From<&'a D>
  {
    self
      .cache
      .remove(details.into())
      .ok_or(format!("Could not find resource {:?}", details))
  }

  /// Give a texture to the manager.
  pub fn put_resource<D>(&mut self, details: &D, resource: R)
  where
    L: ResourceLoader<'l, R, Args = D>,
    D: Debug + Eq + Hash + ?Sized,
    K: Borrow<D> + for<'a> From<&'a D>
  {
    self
      .cache
      .insert(details.into(), resource);
  }
}


pub type TextureManager<'l, T> = ResourceManager<'l, String, Texture<'l>, TextureCreator<T>>;


pub type FontManager<'l> = ResourceManager<'l, FontDetails, Font<'l, 'static>, Sdl2TtfContext>;


pub struct Sdl2Resources<'l> {
  pub texture_creator: &'l TextureCreator<WindowContext>,
  pub texture_manager: TextureManager<'l, WindowContext>,
  pub font_manager: FontManager<'l>,
  pub font_directory: String
}


impl<'l> Sdl2Resources<'l> {
  pub fn new(
    texture_creator: &'l TextureCreator<WindowContext>,
    ttf_ctx: &'l Sdl2TtfContext,
    font_directory: &str
  ) -> Sdl2Resources<'l> {
    // Create the texture manager
    let texture_manager =
      TextureManager::new(texture_creator);
    // Create the font manager
    let font_manager =
      FontManager::new(ttf_ctx);

    Sdl2Resources {
      texture_creator,
      texture_manager,
      font_manager,
      font_directory: font_directory.to_string()
    }
  }
}
