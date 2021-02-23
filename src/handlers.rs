use super::{Container, DbPoolGetter};
use crate::middleware::Role;
use crate::models::{self, NewUser, User, UserError};
use darpi::job::{IOBlockingJob, SenderExt};
use darpi::{chrono::Duration, from_path, handler, Json, Query};
use darpi_middleware::{auth::*, body_size_limit};
use log::warn;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;
use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug)]
pub struct Login {
    email: String,
    password: String,
}

// here we give the container type
// so the framework knows where to get
// the requested `Arc<dyn JwtTokenCreator>` from
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

// the `from_path` attribute allows us
// to deserialize `UserID` from the request path
#[from_path]
#[derive(Deserialize, Serialize, Debug, Query)]
pub struct Name {
    name: String,
}

#[handler]
pub(crate) async fn home() -> String {
    "Welcome to darpi".to_string()
}

// here we give the container type
// so the framework knows where to get
// the requested `Arc<dyn DbPoolGetter>` from
// enforce max request body size 128 bytes and admin role via middleware
#[handler({
    container: Container,
    middleware: {
        request: [body_size_limit(128), authorize(Role::Admin)]
    }
})]
pub(crate) async fn create_user(
    #[body] new_user: Json<NewUser>,
    #[inject] db_pool: Arc<dyn DbPoolGetter>,
    #[blocking] job_queue: Sender<IOBlockingJob>,
) -> Result<Json<User>, UserError> {
    let conn = db_pool.pool().get()?;

    //diesel does not have an async api
    //we don't want to block the server thread
    //so we will offload this as a blocking task
    // to be executed on an appropriate thread
    // and we will wait for the result on an async channel
    let user = job_queue
        .oneshot(move || models::create_user(new_user.into_inner(), &conn))
        .await
        .map_err(|_| UserError::InternalError)?
        .await
        .map_err(|_| UserError::InternalError)??;

    Ok(Json(user))
}

// the `from_path` attribute allows us
// to deserialize `UserID` from the request path
#[from_path]
#[derive(Deserialize)]
pub(crate) struct UserID {
    id: i32,
}

// here we give the container type
// so the framework knows where to get
// the requested `Arc<dyn DbPoolGetter>` from
#[handler({
    container: Container
})]
pub(crate) async fn get_user(
    #[path] user_id: UserID,
    #[inject] db_pool: Arc<dyn DbPoolGetter>,
    #[blocking] job_queue: Sender<IOBlockingJob>,
) -> Result<Option<Json<User>>, UserError> {
    let conn = db_pool.pool().get()?;

    //diesel does not have an async api
    //we don't want to block the server thread
    //so we will offload this as a blocking task
    // to be executed on an appropriate thread
    // and we will wait for the result on an async channel
    let user = job_queue
        .oneshot(move || models::find_user_by_id(user_id.id, &conn))
        .await
        .map_err(|_| UserError::InternalError)?
        .await
        .map_err(|_| UserError::InternalError)??;

    user.map_or(Ok(None), |u| Ok(Some(Json(u))))
}
