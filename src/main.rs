use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde_json::json;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
mod handler;
mod model;
mod schema;
use actix_cors::Cors;

pub struct AppState {
    db: MySqlPool,
}

#[get("/healthchecker")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "Build Crud with Rust , sqlx , and mysql";

    HttpResponse::Ok().json(json!({"status" : "success", "message" : MESSAGE}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if (std::env::var_os("RUST_LOG").is_none()) {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set re");
    let pool = match MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("connected to DB");
            pool
        }
        Err(error) => {
            println!("🔥 Failed to connect to the database: {:?}", error);
            std::process::exit(1);
        }
    };

    println!("🚀 Server started successfully");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .configure(handler::config)
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
