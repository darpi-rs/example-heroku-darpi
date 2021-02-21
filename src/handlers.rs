use super::{Container, DbPoolGetter};
use crate::middleware::Role;
use crate::models::{self, NewUser, User, UserError};
use darpi::{chrono::Duration, from_path, handler, Json, Query};
use darpi_middleware::{auth::*, body_size_limit};
use log::warn;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug)]
pub struct Login {
    email: String,
    password: String,
}

#[handler({
    container: Container
})]
pub(crate) async fn login(
    #[body] _login_data: Json<Login>,
    #[inject] jwt_tok_creator: Arc<dyn JwtTokenCreator>,
) -> Result<Token, Error> {
    let admin = Role::Admin; // hardcoded just for the example
    let uid = "uid"; // hardcoded just for the example
    let tok = jwt_tok_creator
        .create(uid, &admin, Duration::days(30))
        .await
        .map_err(|e| {
            warn!("could not create a token: {}", e);
            e
        })?;

    Ok(tok)
}

#[from_path]
#[derive(Deserialize, Serialize, Debug, Query)]
pub struct Name {
    name: String,
}

#[handler]
pub(crate) async fn home() -> String {
    "Welcome to darpi".to_string()
}

// enforce admin role with authorize middleware
#[handler({
    container: Container,
    middleware: {
        request: [body_size_limit(128), authorize(Role::Admin)]
    }
})]
pub(crate) async fn create_user(
    #[body] new_user: Json<NewUser>,
    #[inject] db_pool: Arc<dyn DbPoolGetter>,
) -> Result<Json<User>, UserError> {
    let conn = db_pool.pool().get()?;

    let user =
        tokio::task::spawn_blocking(move || models::create_user(new_user.into_inner(), &conn))
            .await??;

    Ok(Json(user))
}

#[from_path]
#[derive(Deserialize)]
pub(crate) struct UserID {
    id: i32,
}

#[handler({
    container: Container
})]
pub(crate) async fn get_user(
    #[path] user_id: UserID,
    #[inject] db_pool: Arc<dyn DbPoolGetter>,
) -> Result<Option<Json<User>>, UserError> {
    let conn = db_pool.pool().get()?;

    let user =
        tokio::task::spawn_blocking(move || models::find_user_by_id(user_id.id, &conn)).await??;

    user.map_or(Ok(None), |u| Ok(Some(Json(u))))
}
