// src/main.rs
use actix_web::{App, HttpServer, web};
mod db;
mod handlers;
mod models;
mod repo;
mod schema;

use crate::db::postgres::init_pool;
use crate::handlers::user_handlers::user_handlers_scope;

use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = env::var("dbUrl").expect("DATABASE_URL must be set");
    let pool = init_pool(&database_url);

    println!("listening on port 8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(user_handlers_scope())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
