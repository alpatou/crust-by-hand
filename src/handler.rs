use crate::{
    model::{NoteModel, NoteModelResponse},
    schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
    AppState,
};

use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use serde_json::json;

fn filter_db_record(note: &NoteModel) -> NoteModelResponse {
    NoteModelResponse {
        id: note.id.to_owned(),
        title: note.title.to_owned(),
        content: note.content.to_owned(),
        category: note.category.to_owned().unwrap(),
        published: note.published != 0,
        createdAt: note.created_at.unwrap(),
        updatedAt: note.updated_at.unwrap(),
    }
}

#[get("/notes")]
pub async fn note_list_handler(
    opts: web::Data<FilterOptions>,
    data: web::Data<AppState>,
) -> impl Responder {
    let limit: usize = opts.limit.unwrap_or(10);
    let offset: usize = (opts.page.unwrap_or(1) - 1) * limit;

    HttpResponse::Ok().json("value")
}
