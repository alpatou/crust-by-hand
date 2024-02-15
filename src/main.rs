use actix_web::middleware::Logger;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use serde_json::json;
use sqlx::MySqlPool;
mod handler;
mod model;
mod schema;

pub struct AppState {
    db: MySqlPool,
}

#[get("/api/healthchecker")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "Build Crud with Rust , sqlx , and mysql";

    HttpResponse::Ok().json(json!({"status" : "success", "message" : MESSAGE}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .service(health_checker_handler)
            .service(handler::note_list_handler)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
