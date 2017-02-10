
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct Options {
    pub manifest_path: Option<String>,
    pub packages: Vec<String>,
}

impl Options {
    pub fn new() -> Self {
        Options {
            manifest_path: None,
            packages: Vec::new(),
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Options::new()
    }
}