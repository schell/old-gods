//! Information needed to load a Font


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontDetails {
    pub path: String,
    pub size: u16,
}


impl FontDetails {
    pub fn to_css_string(&self) -> String {
        let s = format!("{}px {}", self.size, self.path);
        s
    }
}


impl<'a> From<&'a FontDetails> for FontDetails {
    fn from(details: &'a FontDetails) -> FontDetails {
        FontDetails {
            path: details.path.clone(),
            size: details.size,
        }
    }
}
