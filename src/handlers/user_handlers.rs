// src/handlers/user_handlers.rs
use actix_web::{Error, HttpResponse, Result, web};

use actix_web::error::ErrorBadRequest;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
use crate::repo::user_repo::{create_user, get_user_by_username, update_last_login};

use crate::models::user_models::*;

use crate::schema::users::dsl::*;
use actix_web::error::ErrorInternalServerError;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;


use uuid::Uuid;


/// Function to configure all user-related routes within the /api scope.
pub fn user_handlers_scope() -> actix_web::Scope {
    web::scope("/api")
        .route("/login", web::post().to(login))
        .route("/users", web::get().to(users_endpoint))
        .route("/signup", web::post().to(sign_up))
}


pub async fn sign_up(
    pool: web::Data<DbPool>,
    payload: web::Json<SignUpReq>,
) -> Result<HttpResponse, Error> {
    // 1 — Validate required fields
    if payload.username.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json("Username cannot be empty"));
    }

    if payload.password.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json("Password cannot be empty"));
    }

    if payload.email.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json("Email cannot be empty"));
    }

    let username_fetched = payload.username.clone();
    let password = payload.password.clone();
    let email_fetched = payload.email.clone();
    let display_name_fetched = payload.display_name.clone();

    // Hash password
    let password_hash_generated = hash(password, DEFAULT_COST).map_err(ErrorInternalServerError)?;

    // Get DB connection
    let mut connection = pool.get().map_err(ErrorInternalServerError)?;

    // Prepare new user struct
    let new_user = NewUser {
        id: Uuid::new_v4(),
        username: username_fetched,
        email: email_fetched,
        password_hash: password_hash_generated,
        display_name: display_name_fetched,
        user_role: "user".into(),
        is_active: true,
        last_login: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // // Insert into DB
    // diesel::insert_into(users)
    //     .values(&new_user)
    //     .execute(&mut connection)
    //     .map_err(ErrorInternalServerError)?;
    create_user(&mut connection, &new_user) // <-- FIXED: Passing &mut connection
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json("User created successfully"))
}

pub async fn login(
    pool: web::Data<DbPool>,
    payload: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(ErrorInternalServerError)?;

    // Extract from request
    let username_input = payload.username.clone();
    let password_input = payload.password.clone();

    // Fetch user by username
    // let user = users
    //     .filter(username.eq(username_input))
    //     .first::<User>(&mut conn)
    //     .map_err(|_| ErrorBadRequest("Invalid username or password"))?;
    let user = get_user_by_username(&mut conn, &username_input)
        .map_err(|_| ErrorBadRequest("Invalid username or password"))?;

    // Ensure password exists
    let stored_hash = user
        .password_hash
        .clone()
        .ok_or_else(|| ErrorBadRequest("User has no password stored"))?;

    // Check password
    let is_valid = verify(password_input, &stored_hash).map_err(ErrorInternalServerError)?;

    if !is_valid {
        return Err(ErrorBadRequest("Invalid username or password"));
    }

    // Update last login
    // diesel::update(users.filter(id.eq(user.id)))
    //     .set(last_login.eq(Utc::now()))
    //     .execute(&mut conn)
    //     .map_err(ErrorInternalServerError)?;
    update_last_login(&mut conn, user.id).map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json("Logged in successfully"))
}


pub async fn users_endpoint(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let mut connection = pool.get().expect("cant get db connection from pool");

    let users_data = web::block(move || users.select(User::as_select()).load(&mut connection))
        .await
        .map_err(ErrorInternalServerError)?
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(users_data))
}

// pub async fn hello() -> impl Responder {
//     HttpResponse::Ok().body("hello there")
// }
