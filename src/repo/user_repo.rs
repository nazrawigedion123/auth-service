use crate::models::user_models::{NewUser, User};
use crate::schema::users;
use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
// use diesel::sql_types::Uuid;
use users::dsl::*; // <-- Keep this at the top for broad use
use uuid::Uuid;

/// Inserts a new user record into the database.
///
/// This function is part of the repository layer, isolating database operations
/// from the application handlers.
///
/// It returns an empty Ok(()) on success, or a Diesel error on failure.
pub fn create_user(
    conn: &mut PgConnection,
    new_user: &NewUser,
) -> Result<(), diesel::result::Error> {
    // REMOVED: use users::dsl::*; // This line was redundant and causing the conflict

    diesel::insert_into(users)
        .values(new_user)
        .execute(conn)
        .map(|_| ()) // map the number of affected rows (usize) to ()
}

pub fn update_last_login(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<(), diesel::result::Error> {
    // REMOVED: use users::dsl::*; // This line was redundant and causing the conflict
    diesel::update(users.filter(id.eq(user_id)))
        .set(last_login.eq(Utc::now()))
        .execute(conn)
        .map(|_| ()) // map the number of affected rows (usize) to ()
}

pub fn get_user_by_username(
    conn: &mut PgConnection,
    target_username: &str,
) -> Result<User, diesel::result::Error> {
    users
        .filter(username.eq(target_username))
        .select(User::as_select())
        .first(conn)
}

/*
users
        .filter(username.eq(username_input))
        .first::<User>(&mut conn)
        .map_err(|_| ErrorBadRequest("Invalid username or password"))?;*/
