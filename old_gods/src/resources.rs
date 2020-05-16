//! Traits and types for loading shared resources.
use log::warn;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wasm_bindgen::{closure::Closure, JsValue};


#[derive(Clone)]
/// The loading status of a resource.
pub enum LoadStatus {
    None,
    Started,
    Complete,
    Error(String),
}


/// A shared resource.
/// All clones of this resource refer to the same underlying memory.
#[derive(Clone)]
pub struct SharedResource<T> {
    payload: Arc<Mutex<(LoadStatus, Option<T>)>>,
}

impl<T> Default for SharedResource<T> {
    fn default() -> Self {
        SharedResource {
            payload: Arc::new(Mutex::new((LoadStatus::None, None))),
        }
    }
}


impl<T> SharedResource<T> {
    pub fn with_payload<A>(&self, f: impl FnOnce(&(LoadStatus, Option<T>)) -> A) -> A {
        let payload = self
            .payload
            .try_lock()
            .expect("Could not acquire lock - load_sprite_sheet::load");
        f(&payload)
    }

    pub fn with_status<A>(&self, f: impl FnOnce(&LoadStatus) -> A) -> A {
        self.with_payload(|p| f(&p.0))
    }

    pub fn with_resource<A>(&self, f: impl FnOnce(&T) -> A) -> Option<A> {
        self.with_payload(|p| p.1.as_ref().map(f))
    }

    pub fn set_status(&self, status: LoadStatus) {
        let mut payload = self
            .payload
            .try_lock()
            .expect("Could not acquire lock - load_sprite_sheet::load");
        payload.0 = status;
    }

    pub fn set_resource(&self, may_rsrc: Option<T>) {
        let mut payload = self
            .payload
            .try_lock()
            .expect("Could not acquire lock - load_sprite_sheet::load");
        payload.1 = may_rsrc;
    }

    pub fn set_status_and_resource(&self, new_payload: (LoadStatus, Option<T>)) {
        let mut payload = self
            .payload
            .try_lock()
            .expect("Could not acquire lock - load_sprite_sheet::load");
        *payload = new_payload;
    }
}


/// A generic way to load resources.
/// Resources loaded this way can be polled in subsystems.
pub trait Resources<R> {
    fn status_of(&self, key: &str) -> LoadStatus;
    fn load(&mut self, key: &str);
    fn take(&mut self, key: &str) -> Option<SharedResource<R>>;
    fn put(&mut self, key: &str, rsrc: SharedResource<R>);
}


/// Poll the load status of a resource:
/// * if it has not yet started a load, start a new loading process, return nothing
/// * if it is loading, do nothing, return nothing
/// * if it is complete call the closure with the resource and return some answer
/// * if it has erred, return the error message
pub fn when_loaded<Rs, R, F, T>(rs: &mut Rs, key: &str, f: F) -> Result<Option<T>, String>
where
    Rs: Resources<R>,
    F: FnOnce(&R) -> T,
{
    match rs.status_of(&key) {
        LoadStatus::None => {
            // Load it and come back later
            rs.load(&key);
            return Ok(None);
        }
        LoadStatus::Started => {
            // Come back later because it's loading etc.
            return Ok(None);
        }
        LoadStatus::Complete => {}
        LoadStatus::Error(msg) => {
            warn!("sprite sheet loading error: {}", msg);
            return Err(msg);
        }
    }

    if let Some(shared) = rs.take(key) {
        let may_t: Option<T> = shared.with_resource(|r| f(r));
        rs.put(key, shared);
        Ok(may_t)
    } else {
        Err("No shared resource - this should not happen".to_string())
    }
}


/// Helps hold JS closures while loading resources.
pub struct Callbacks {
    pub loading: Arc<Closure<dyn Fn(JsValue)>>,
    pub error: Arc<Closure<dyn Fn(JsValue)>>,
}


impl Callbacks {
    pub fn new(loading: Closure<dyn Fn(JsValue)>, error: Closure<dyn Fn(JsValue)>) -> Self {
        Callbacks {
            loading: Arc::new(loading),
            error: Arc::new(error),
        }
    }
}


/// Helper function for writing [Resources::status_of] for types that contain
/// a hashmap of shared resources.
pub fn status_of_sharedmap<T>(
    resources: &HashMap<String, SharedResource<T>>,
    s: &str,
) -> LoadStatus {
    resources
        .get(s)
        .map(|rsrc| rsrc.with_status(|s| s.clone()))
        .unwrap_or(LoadStatus::None)
}


/// An inner type for loadable resources.
///
/// This struct's implementation of [Resources::load] does nothing, but
/// a wrapper type can proxy the other trait functions to an inner
/// LoadableResources<T> and implement its own [Resources::load] function.
/// This cuts down on the boilerplate of making new resources.
///
/// See [old_gods::image::HtmlImageResources] for an example of this.
pub struct LoadableResources<T> {
    pub resources: HashMap<String, SharedResource<T>>,
    pub callbacks: HashMap<String, Callbacks>,
}


impl<T> Default for LoadableResources<T> {
    fn default() -> Self {
        Self::new()
    }
}


impl<T> LoadableResources<T> {
    pub fn new() -> Self {
        LoadableResources {
            resources: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }
}


impl<T: Clone> Resources<T> for LoadableResources<T> {
    fn status_of(&self, s: &str) -> LoadStatus {
        status_of_sharedmap(&self.resources, s)
    }

    fn load(&mut self, _path: &str) {
        // TODO: Think about making [Resources::load] return a Result<(), String>
        panic!("Attempting to use LoadableResources::load when this type is meant to be wrapped!");
    }

    fn take(&mut self, s: &str) -> Option<SharedResource<T>> {
        if self.callbacks.contains_key(s) {
            let _ = self.callbacks.remove(s);
        }
        self.resources.remove(s)
    }

    fn put(&mut self, path: &str, shared: SharedResource<T>) {
        self.resources.insert(path.to_string(), shared);
    }
}
