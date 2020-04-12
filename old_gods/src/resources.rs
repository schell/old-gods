//! Traits ad types for loading shared resources.
use log::{trace, warn};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, EventTarget, HtmlImageElement};


#[derive(Clone)]
/// The loading status of a resource.
pub enum LoadStatus {
    None,
    Started,
    Complete,
    Error(String),
}


pub trait Resources<R> {
    fn status_of(&self, key: &str) -> LoadStatus;
    fn load(&mut self, key: &str);
    fn take(&mut self, key: &str) -> Option<R>;
    fn put(&mut self, key: &str, rsrc: R);

    /// Poll the load status of a resource:
    /// * if it has not yet started a load, start a new loading process, return nothing
    /// * if it is loading, do nothing, return nothing
    /// * if it is complete call the closure with the resource and return some answer
    /// * if it has erred, return the error message
    fn when_loaded<F, T>(&mut self, key: &str, f: F) -> Result<Option<T>, String>
    where
        F: FnOnce(&R) -> T,
    {
        match self.status_of(&key) {
            LoadStatus::None => {
                // Load it and come back later
                self.load(&key);
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

        let rsrc = self.take(&key).expect("Could not take sprite sheet");
        let t = f(&rsrc);
        self.put(&key, rsrc);
        Ok(Some(t))
    }
}


pub struct Callbacks(Arc<Closure<dyn Fn(JsValue)>>, Arc<Closure<dyn Fn(JsValue)>>);


pub struct HtmlResources {
    sprite_sheets: HashMap<String, Arc<Mutex<(LoadStatus, Option<HtmlImageElement>)>>>,
    callbacks: HashMap<String, Callbacks>,
}


impl Default for HtmlResources {
    fn default() -> Self {
        Self::new()
    }
}


impl HtmlResources {
    pub fn new() -> Self {
        HtmlResources {
            sprite_sheets: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }
}


impl Resources<HtmlImageElement> for HtmlResources {
    fn status_of(&self, s: &str) -> LoadStatus {
        self.sprite_sheets
            .get(s)
            .map(|payload| {
                let status_and_may_img = payload.try_lock().unwrap();
                status_and_may_img.0.clone()
            })
            .unwrap_or(LoadStatus::None)
    }

    fn load(&mut self, path: &str) {
        trace!("loading sprite sheet: {}", path);
        let img = window()
            .expect("no window")
            .document()
            .expect("no document")
            .create_element("img")
            .expect("can't create img")
            .dyn_into::<HtmlImageElement>()
            .expect("can't coerce img");
        img.set_src(path);
        let status = Arc::new(Mutex::new((LoadStatus::Started, Some(img.clone()))));
        let target: &EventTarget = img.dyn_ref().expect("can't coerce img as EventTarget");
        let load_status = status.clone();
        let load_path = path.to_string();
        let load = Closure::wrap(Box::new(move |_: JsValue| {
            let mut status_and_img = load_status
                .try_lock()
                .expect("Could not acquire lock - load_sprite_sheet::load");
            trace!("  loading {} complete", &load_path);
            status_and_img.0 = LoadStatus::Complete;
        }) as Box<dyn Fn(JsValue)>);
        let err_status = status.clone();
        let err_path = path.to_string();
        let err = Closure::wrap(Box::new(move |event: JsValue| {
            let mut status_and_img = err_status
                .try_lock()
                .expect("Could not acquire lock - load_sprite_sheet::err");
            trace!("error event: {:#?}", event);
            let event = event
                .dyn_into::<web_sys::Event>()
                .expect("Error is not an Event");
            let msg = format!("failed loading {}: {}", &err_path, event.type_());
            trace!("  loading {} erred: {}", &err_path, &msg);
            status_and_img.0 = LoadStatus::Error(msg);
            status_and_img.1 = None;
        }) as Box<dyn Fn(JsValue)>);
        target
            .add_event_listener_with_callback("load", load.as_ref().unchecked_ref())
            .unwrap();
        target
            .add_event_listener_with_callback("error", err.as_ref().unchecked_ref())
            .unwrap();
        self.callbacks
            .insert(path.to_string(), Callbacks(Arc::new(load), Arc::new(err)));
        self.sprite_sheets.insert(path.to_string(), status);
    }

    fn take(&mut self, s: &str) -> Option<HtmlImageElement> {
        let _ = self.callbacks.remove(s);
        let status_and_img = self.sprite_sheets.remove(s)?;
        let status_and_img = status_and_img.try_lock().ok()?;
        status_and_img.1.clone()
    }

    fn put(&mut self, path: &str, tex: HtmlImageElement) {
        let status_and_img = Arc::new(Mutex::new((LoadStatus::Complete, Some(tex))));
        self.sprite_sheets.insert(path.to_string(), status_and_img);
    }
}
