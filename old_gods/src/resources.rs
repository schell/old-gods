//! Traits ad types for loading shared resources.
use log::warn;


#[derive(Clone)]
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
