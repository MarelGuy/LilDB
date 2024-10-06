use super::document::Document;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Collection {
    pub name: String,
    pub path: String,
    pub created_at: String,
    documents: Vec<Document>,
}

impl Collection {
    pub fn new(name: String, path: String, documents: Vec<Document>) -> Self {
        Self {
            name,
            path,
            created_at: chrono::prelude::Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            documents,
        }
    }
}
