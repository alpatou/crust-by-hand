use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateNoteSchema {
    pub title: String,
    pub content: String,
    #[serde(skip_serialization_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serialization_if = "Option::is_none")]
    pub published: Option<Bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateNoteSchema {
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub published: Option<Bool>,
}
