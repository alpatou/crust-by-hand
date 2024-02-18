use crate::{
    health_checker_handler,
    model::{NoteModel, NoteModelResponse},
    schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
    AppState,
};
use actix_web::{
    delete, get,
    guard::Not,
    patch, post,
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};
use serde::ser::Impossible;
use serde_json::Value;
use std::fmt::Error;

pub fn config(conf: &mut web::ServiceConfig) -> () {
    let scope = web::scope("/api")
        .service(health_checker_handler)
        .service(note_list_handler)
        .service(edit_note_handler)
        .service(create_note_handler)
        .service(create_note_handler)
        .service(delete_note_handler);

    conf.service(scope);
}

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

#[patch("/api/notes/{id}")]
async fn edit_note_handler(
    path: web::Path<uuid::Uuid>,
    body: web::Json<UpdateNoteSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    // fetch the note
    let note_id: String = path.into_inner().to_string();

    let query_result = sqlx::query_as!(NoteModel, r#"SELECT * FROM notes WHERE id=? "#, note_id)
        .fetch_one(&data.db)
        .await;

    let note = match query_result {
        Ok(note) => note,
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().json(
                serde_json::json!({"status": "fail","message": format!("Note with ID: {} not found", note_id)}),
            )
        },
        Err(error) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"status" : "error", "message" : format!("{}", error)})
            );
        }
    };

    // process the body

    let published: bool = body.published.unwrap_or(note.published != 0);

    let i8_published: i8 = published as i8;

    let update_result = sqlx::query(
        r#"UPDATE notes set title = ? , content = ? category = ? , published = ? WHERE id = ?"#,
    )
    .bind(body.title.to_owned())
    .bind(body.content.to_owned())
    .bind(body.category.to_owned())
    .bind(i8_published)
    .bind(note_id.to_owned())
    .execute(&data.db)
    .await;

    match update_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let message = format!("Note with ID: {} not found", note_id);
                return HttpResponse::NotFound()
                    .json(serde_json::json!({"status": "fail","message": message}));
            }
        }
        Err(e) => {
            let message = format!("Internal server error: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": message}));
        }
    }

    // refetch it
    let updated_note = sqlx::query_as!(
        NoteModel,
        r#"SELECT * FROM notes WHERE id=?"#,
        note_id.to_owned()
    )
    .fetch_one(&data.db)
    .await;

    return match updated_note {
        Ok(note) => {
            let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "note": filter_db_record(&note)
            })});

            HttpResponse::Ok().json(note_response)
        }
        Err(error) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error","message": format!("{:?}", error)})),
    };
}

#[delete("/notes/{id}")]
async fn delete_note_handler(
    path: web::Path<uuid::Uuid>,
    data: web::Data<AppState>,
) -> impl Responder {
    let note_id = path.into_inner().to_string();
    let query_result = sqlx::query!(r#"DELETE FROM notes WHERE id = ?"#, note_id)
        .execute(&data.db)
        .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let message = format!("Note with ID: {} not found", note_id);
                HttpResponse::NotFound()
                    .json(serde_json::json!({"status": "fail","message": message}))
            } else {
                HttpResponse::NoContent().finish()
            }
        }
        Err(e) => {
            let message = format!("Internal server error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": message}))
        }
    }
}
