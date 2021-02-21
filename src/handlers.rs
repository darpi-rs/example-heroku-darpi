use super::Container;
use crate::middleware::{roundtrip, Role};
use darpi::chrono::Duration;
use darpi::{handler, Json, Path};
use darpi_middleware::auth::*;
use darpi_middleware::body_size_limit;
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

#[derive(Deserialize, Serialize, Debug, Path)]
pub struct Name {
    name: String,
}

#[handler({
    container: Container,
    middleware: {
        request: [body_size_limit(90), roundtrip("blah")]
    }
})]
pub(crate) async fn home(#[middleware::request(1)] m_str: String) -> String {
    format!("home {}", m_str)
}

#[handler({
    container: Container,
    middleware: {
        request: [body_size_limit(64)]
    }
})]
pub(crate) async fn do_something(#[path] p: Name) -> String {
    format!("user {}", p.name)
}

// enforce admin role with authorize middleware
#[handler({
    container: Container,
    middleware: {
        request: [body_size_limit(128), authorize(Role::Admin)]
    }
})]
pub(crate) async fn important(#[path] p: Name) -> String {
    format!("user token {}", p.name)
}

#[handler({
    middleware: {
        request: [roundtrip("blah")]
    }
})]
async fn do_something123(
    // the request query is deserialized into Name
    // if deseriliazation fails, it will result in an error response
    // to make it optional wrap it in an Option<Name>
    #[query] query: Name,
    // the request path is deserialized into Name
    #[path] path: Name,
    // the request body is deserialized into the struct Name
    // it is important to mention that the wrapper around Name
    // should implement darpi::request::FromRequestBody
    // Common formats like Json, Xml and Yaml are supported out
    // of the box but users can implement their own
    #[body] payload: Json<Name>,
    // we can access the T from Ok(T) in the middleware result
    #[middleware::request(0)] m_str: String, // returning a String works because darpi has implemented
                                             // the Responder trait for common types
) -> String {
    format!(
        "query: {} path: {} body: {} middleware: {}",
        query.name, path.name, payload.name, m_str
    )
}
