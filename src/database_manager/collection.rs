#[derive(Clone, Debug)]
pub struct Collection {
    name: String,
    path: String,
    documents: Vec<String>,
}

impl Collection {
    pub fn new(name: String, path: String, documents: Vec<String>) -> Self {
        Self {
            name,
            path,
            documents,
        }
    }
}
