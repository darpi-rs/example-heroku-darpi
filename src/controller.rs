use super::Container;
use crate::middleware::{roundtrip, Role};
use darpi::{handler, Json, Path};
use darpi_middleware::auth::*;
use darpi_middleware::body_size_limit;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug)]
pub struct Login {
    email: String,
    password: String,
}

#[handler(Container, [roundtrip("hello")])]
pub(crate) async fn login(
    // deser the request body json login
    #[body] _login_data: Json<Login>,
    // we get this from shaku DI container
    #[inject] jwt_tok_creator: Arc<dyn JwtTokenCreator>,
    // we access the middleware Ok(T) by accessing the index of the specific middleware above
    #[middleware(0)] msg: String,
) -> Result<Token, Error> {
    // we gave the &str "hello" to the roundtrip middleware
    // then it appended its own message and returned a String to us
    // anecdotal example but it shows how we can communicate between
    // handlers and request middleware
    println!("{}", msg);
    let admin = Role::Admin; // hardcoded just for the example
    let uid = "uid"; // hardcoded just for the example
    let tok = jwt_tok_creator.create(uid, &admin).await?;
    Ok(tok)
}

#[derive(Deserialize, Serialize, Debug, Path)]
pub struct Name {
    name: String,
}

// enforce admin role with authorize middleware
#[handler(Container, [body_size_limit(64), authorize(Role::Admin)])]
pub(crate) async fn do_something(#[path] p: Name, #[middleware(1)] _token: Token) -> String {
    format!("user token {}", p.name)
}
