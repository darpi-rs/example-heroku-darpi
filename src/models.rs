use crate::schema::users;
use darpi::response::ResponderError;
use derive_more::Display;
use diesel::prelude::*;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::result::Error as DieselError;
use diesel::{ExpressionMethods, Insertable, Queryable};
use diesel::{PgConnection, RunQueryDsl};
use r2d2::Error as R2D2Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Insertable, Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Display)]
pub enum UserError {
    DBError(R2D2Error),
    InsertError(DieselError),
    TokioError(tokio::task::JoinError),
    InternalError
}

impl From<R2D2Error> for UserError {
    fn from(e: R2D2Error) -> Self {
        Self::DBError(e)
    }
}

impl From<DieselError> for UserError {
    fn from(e: DieselError) -> Self {
        Self::InsertError(e)
    }
}

impl From<tokio::task::JoinError> for UserError {
    fn from(e: tokio::task::JoinError) -> Self {
        Self::TokioError(e)
    }
}

impl ResponderError for UserError {}

pub fn create_user(
    // prevent collision with `name` column imported inside the function
    new_user: NewUser,
    conn: &PgConnection,
) -> Result<User, DieselError> {
    use crate::schema::users::dsl::*;

    let new_user = diesel::insert_into(users)
        .values(new_user)
        .get_result(conn)?;

    Ok(new_user)
}

pub fn find_user_by_id(user_id: i32, conn: &PgConnection) -> Result<Option<User>, DieselError> {
    use crate::schema::users::dsl::*;

    let user = FilterDsl::filter(users, id.eq(user_id))
        .first::<User>(conn)
        .optional()?;

    Ok(user)
}
