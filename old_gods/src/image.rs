//! Loading images/textures.
use log::trace;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, EventTarget, HtmlImageElement};

use super::prelude::{Callbacks, LoadStatus, LoadableResources, Resources, SharedResource};


pub struct HtmlImageResources(pub LoadableResources<HtmlImageElement>);


impl Default for HtmlImageResources {
    fn default() -> Self {
        Self::new()
    }
}


impl HtmlImageResources {
    pub fn new() -> Self {
        HtmlImageResources(LoadableResources::new())
    }
}


impl Resources<HtmlImageElement> for HtmlImageResources {
    fn status_of(&self, s: &str) -> LoadStatus {
        self.0.status_of(s)
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

        let rsrc = SharedResource::default();
        rsrc.set_status_and_resource((LoadStatus::Started, Some(img.clone())));

        let load_rsrc = rsrc.clone();
        let load = Closure::wrap(Box::new(move |_: JsValue| {
            load_rsrc.set_status(LoadStatus::Complete);
        }) as Box<dyn Fn(JsValue)>);

        let err_rsrc = rsrc.clone();
        let err_path = path.to_string();
        let err = Closure::wrap(Box::new(move |event: JsValue| {
            trace!("error event: {:#?}", event);
            let event = event
                .dyn_into::<web_sys::Event>()
                .expect("Error is not an Event");
            let msg = format!("failed loading {}: {}", &err_path, event.type_());
            trace!("  loading {} erred: {}", &err_path, &msg);
            err_rsrc.set_status_and_resource((LoadStatus::Error(msg), None));
        }) as Box<dyn Fn(JsValue)>);

        let target: &EventTarget = img.dyn_ref().expect("can't coerce img as EventTarget");
        target
            .add_event_listener_with_callback("load", load.as_ref().unchecked_ref())
            .unwrap();
        target
            .add_event_listener_with_callback("error", err.as_ref().unchecked_ref())
            .unwrap();
        self.0
            .callbacks
            .insert(path.to_string(), Callbacks::new(load, err));
        self.0.resources.insert(path.to_string(), rsrc);
    }

    fn take(&mut self, s: &str) -> Option<SharedResource<HtmlImageElement>> {
        self.0.take(s)
    }

    fn put(&mut self, path: &str, shared_tex: SharedResource<HtmlImageElement>) {
        self.0.put(path, shared_tex)
    }
}
