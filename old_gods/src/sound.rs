use super::prelude::{Callbacks, LoadStatus, LoadableResources, Resources, SharedResource};
use log::trace;
use std::collections::HashMap;
use wasm_bindgen::{
    prelude::{Closure, JsValue},
    JsCast,
};
use web_sys::{window, AudioContext, EventTarget, HtmlAudioElement, MediaElementAudioSourceNode};


pub struct SoundBlaster {
    context: AudioContext,
    resources: LoadableResources<HtmlAudioElement>,
    tracks: HashMap<String, MediaElementAudioSourceNode>,
}


impl SoundBlaster {
    pub fn new() -> Self {
        SoundBlaster {
            context: AudioContext::new().expect("Could not create an AudioContext"),
            resources: LoadableResources::new(),
            tracks: HashMap::new(),
        }
    }
}


impl Default for SoundBlaster {
    fn default() -> Self {
        Self::new()
    }
}


impl Resources<HtmlAudioElement> for SoundBlaster {
    fn status_of(&self, s: &str) -> LoadStatus {
        self.resources.status_of(s)
    }

    fn load(&mut self, path: &str) {
        trace!("loading sound: {}", path);

        let audio_element = window()
            .expect("no window")
            .document()
            .expect("no document")
            .create_element("audio")
            .expect("can't create audio element")
            .dyn_into::<HtmlAudioElement>()
            .expect("can't coerce audio element");
        audio_element.set_src(path);

        // https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Using_Web_Audio_API
        let track = self
            .context
            .create_media_element_source(&audio_element)
            .unwrap();
        track
            .connect_with_audio_node(&self.context.destination())
            .unwrap();
        self.tracks.insert(path.to_string(), track);

        let rsrc = SharedResource::new();
        rsrc.set_status_and_resource((LoadStatus::Started, Some(audio_element.clone())));

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

        let target: &EventTarget = audio_element
            .dyn_ref()
            .expect("can't coerce img as EventTarget");
        target
            .add_event_listener_with_callback("load", load.as_ref().unchecked_ref())
            .unwrap();
        target
            .add_event_listener_with_callback("error", err.as_ref().unchecked_ref())
            .unwrap();
        self.resources
            .callbacks
            .insert(path.to_string(), Callbacks::new(load, err));
        self.resources.resources.insert(path.to_string(), rsrc);
    }

    fn take(&mut self, s: &str) -> Option<SharedResource<HtmlAudioElement>> {
        self.resources.take(s)
    }

    fn put(&mut self, path: &str, sound: SharedResource<HtmlAudioElement>) {
        self.resources.put(path, sound)
    }
}
