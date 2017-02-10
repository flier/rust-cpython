#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct Options {
    pub manifest_path: Option<String>,
    pub packages: Vec<String>,
    pub whitelist_types: Vec<String>,
    pub whitelist_fields: Vec<String>,
    pub whitelist_methods: Vec<String>,
    pub blacklist_types: Vec<String>,
    pub blacklist_fields: Vec<String>,
    pub blacklist_methods: Vec<String>,
}

impl Options {
    pub fn new() -> Self {
        Options {
            manifest_path: None,
            packages: Vec::new(),
            whitelist_types: Vec::new(),
            whitelist_fields: Vec::new(),
            whitelist_methods: Vec::new(),
            blacklist_types: Vec::new(),
            blacklist_fields: Vec::new(),
            blacklist_methods: Vec::new(),
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Options::new()
    }
}