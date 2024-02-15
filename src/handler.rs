use std::fmt::Error;

use crate::{
    model::{NoteModel, NoteModelResponse},
    schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
    AppState,
};

use actix_web::{delete, get, guard::Not, patch, post, web, HttpResponse, Responder};
use serde::ser::Impossible;
use serde_json::Value;

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

    let notes: Vec<NoteModel> = sqlx::query_as!(
        NoteModel,
        r#"SELECT * FROM notes ORDER by id LIMIT ? OFFSET ?"#,
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await
    .unwrap();

    println!("{:?}", notes);

    let note_responses = notes
        .into_iter()
        .map(|note| filter_db_record(&note))
        .collect::<Vec<NoteModelResponse>>();

    let json_response: Value = serde_json::json!(
        {
            "status": "success",
            "results" : note_responses.len(),
            "notes": note_responses
        }
    );

    HttpResponse::Ok().json(json_response)
}

#[post("/notes/")]
async fn create_note_handler(
    body: web::Json<CreateNoteSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id: String = uuid::Uuid::new_v4().to_string();
    let query_result =
        sqlx::query(r#"INSERT INTO notes (id, title, content, category) VALUES (?,?,?,?)"#)
            .bind(user_id.clone())
            .bind(body.title.to_string())
            .bind(body.content.to_string())
            .bind(body.category.to_owned().unwrap_or_default())
            .execute(&data.db)
            .await
            .map_err(|err: sqlx::Error| err.to_string());

    // error case
    if let Err(err) = query_result {
        if err.contains("Duplicate Entry") {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"status": "fail", "message": "this already exists"}));
        }

        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error","message": format!("{:?}", err)}));
    }

    let query_result: Result<NoteModel, sqlx::Error> =
        sqlx::query_as!(NoteModel, r#"SELECT * FROM notes WHERE id = ?"#, user_id)
            .fetch_one(&data.db)
            .await;

    match query_result {
        Ok(result) => {
            let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "note": filter_db_record(&result)
            })});

            return HttpResponse::Ok().json(note_response);
        }
        Err(err) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": format!("{:?}", err)}))
        }
    }
}

#[get("/notes/{id}")]
async fn get_note_handler(
    path: web::Path<uuid::Uuid>,
    data: web::Data<AppState>,
) -> impl Responder {
    let note_id = path.into_inner().to_string();
    let query_result = sqlx::query_as!(NoteModel, r#"SELECT * FROM notes WHERE id = ?"#, note_id)
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(result) => {
            let response = serde_json::json!({
                "status": "success",
                "data": serde_json::json!({"note": filter_db_record(&result)})
            });
            return HttpResponse::Ok().json(response);
        }
        Err(_) => todo!(),
    }
}
