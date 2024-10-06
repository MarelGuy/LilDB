#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Document {
    pub name: String,
    pub path: String,
}

impl Document {
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
    }
}
