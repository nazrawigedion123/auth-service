// Bring in all necessary dependencies
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::{Error, HttpResponse,  Result, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;
use diesel::RunQueryDsl;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use uuid::Uuid;

// Imports for Utoipa and Tracing
use tracing::{error, info, instrument};

// Define the type alias for the connection pool
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Internal imports (assuming these paths are correct in your project structure)
use crate::models::user_models::*;
use crate::repo::user_repo::{create_user, get_user_by_username, update_last_login};
use crate::schema::users::dsl::*;



/// Function to configure all user-related routes within the /api scope.
pub fn user_handlers_scope() -> actix_web::Scope {
    web::scope("/api")
        .route("/login", web::post().to(login))
        .route("/users", web::get().to(users_endpoint))
        .route("/signup", web::post().to(sign_up))
}

// --- HANDLER FUNCTIONS ---

// Added #[instrument] for automatic logging of function entry/exit and arguments
#[instrument(skip(pool, payload))]
pub async fn sign_up(
    pool: web::Data<DbPool>,
    payload: web::Json<SignUpReq>,
) -> Result<HttpResponse, Error> {
    // 1 — Validate required fields
    if payload.username.trim().is_empty() {
        info!(username = %payload.username, "Attempted signup with empty username.");
        return Ok(HttpResponse::BadRequest().json("Username cannot be empty"));
    }
    if payload.password.trim().is_empty() {
        info!(username = %payload.username, "Attempted signup with empty password.");
        return Ok(HttpResponse::BadRequest().json("Password cannot be empty"));
    }
    if payload.email.trim().is_empty() {
        info!(username = %payload.username, "Attempted signup with empty email.");
        return Ok(HttpResponse::BadRequest().json("Email cannot be empty"));
    }

    let username_fetched = payload.username.clone();
    let password = payload.password.clone();
    let email_fetched = payload.email.clone();
    let display_name_fetched = payload.display_name.clone();

    // Hash password
    let password_hash_generated = hash(password, DEFAULT_COST).map_err(|e| {
        error!(error = ?e, "Failed to hash password for user: {}", username_fetched);
        ErrorInternalServerError(e)
    })?;

    // Get DB connection
    let mut connection = pool.get().map_err(|e| {
        error!(error = ?e, "Failed to get DB connection for signup.");
        ErrorInternalServerError(e)
    })?;

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

    // Insert into DB using the repository function
    create_user(&mut connection, &new_user).map_err(|e| {
        error!(error = ?e, "Failed to insert user {} into database.", new_user.username);
        ErrorInternalServerError(e)
    })?;

    info!(user_id = %new_user.id, username = %new_user.username, "User registered successfully.");

    Ok(HttpResponse::Ok().json("User created successfully"))
}

// Added #[instrument] and logging the username field for better tracing
#[instrument(skip(pool, payload), fields(username = %payload.username))]
pub async fn login(
    pool: web::Data<DbPool>,
    payload: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    let username_input = payload.username.clone();

    let mut conn = pool.get().map_err(|e| {
        error!(error = ?e, "Failed to get DB connection for login.");
        ErrorInternalServerError(e)
    })?;

    // Extract from request
    let password_input = payload.password.clone();

    // Fetch user by username
    let user = get_user_by_username(&mut conn, &username_input)
        .map_err(|e| { // Log user not found/DB error
            info!(error = ?e, username = %username_input, "Login attempt failed: user not found or DB error.");
            ErrorBadRequest("Invalid username or password")
        })?;

    // Ensure password exists
    let stored_hash = user
        .password_hash
        .clone()
        .ok_or_else(|| { // Log missing password hash
            error!(user_id = %user.id, username = %user.username, "User account is missing password hash.");
            ErrorBadRequest("User has no password stored")
        })?;

    // Check password
    let is_valid = verify(password_input, &stored_hash).map_err(|e| {
        error!(error = ?e, user_id = %user.id, "Internal error during password verification.");
        ErrorInternalServerError(e)
    })?;

    if !is_valid {
        info!(user_id = %user.id, username = %user.username, "Login attempt failed: incorrect password."); // Log invalid password
        return Err(ErrorBadRequest("Invalid username or password"));
    }

    // Update last login
    update_last_login(&mut conn, user.id).map_err(|e| {
        error!(error = ?e, user_id = %user.id, "Failed to update last login timestamp."); // Log update failure
        ErrorInternalServerError(e)
    })?;

    info!(user_id = %user.id, username = %user.username, "User logged in successfully."); // Success log

    Ok(HttpResponse::Ok().json("Logged in successfully"))
}

// Added #[instrument]
#[instrument(skip(pool))]
pub async fn users_endpoint(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let mut connection = pool.get().map_err(|e| {
        // FIX: Use map_err for consistent error handling and logging
        error!(error = ?e, "Failed to get DB connection for fetching users.");
        ErrorInternalServerError(e)
    })?;

    // NOTE: web::block is used to run synchronous database operations on a separate thread
    let users_data: Vec<User> =
        web::block(move || users.select(User::as_select()).load(&mut connection))
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to execute database block for fetching users.");
                ErrorInternalServerError(e)
            })?
            .map_err(|e| {
                error!(error = ?e, "Failed to load users from database.");
                ErrorInternalServerError(e)
            })?;

    info!(
        user_count = users_data.len(),
        "Successfully fetched list of users."
    ); // Success log

    Ok(HttpResponse::Ok().json(users_data)) // FIX: return user_responses, not users_data
}

// #[utoipa::path(
//     get,
//     path = "/hello",
//     responses(
//         (status = 200, description = "Says hello", body = String)
//     )
// )]
// pub async fn hello() -> impl Responder {
//     HttpResponse::Ok().body("Hello world!")
// }
