use crate::extractors::UserExtractor;
use darpi::{middleware, response::ResponderError, Body, RequestParts};
use derive_more::{Display, From};
use std::sync::Arc;

#[allow(unused)]
#[derive(Debug, Display, From)]
pub enum Error {
    #[display(fmt = "no auth header")]
    NoAuthHeaderError,
    #[display(fmt = "Access denied")]
    AccessDenied,
}

impl ResponderError for Error {}

#[allow(unused)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum UserRole {
    Regular,
    Admin,
}

// there are 2 types of middleware `Request` and `Response`
// the constant argument that needs to be present is &RequestParts
// everything else is up to the user
// Arc<dyn UserExtractor> types are injected from the shaku container
// Expect<UserRole> is a special type that is provided by the user when
// the middleware is linked to a handler. This allows the expected value
// to be different per handler + middleware
// middlewares are obgligated to return Result<(), impl ResponderErr>
// if a middleware returns an Err(e) all work is aborted and the coresponding
// response is sent to the user
#[middleware(Request)]
pub async fn access_control(
    #[inject] user_role_extractor: Arc<dyn UserExtractor>,
    #[request_parts] p: &RequestParts,
    #[handler] expected_role: UserRole,
) -> Result<(), Error> {
    let actual_role = user_role_extractor.extract(p).await?;

    if expected_role > actual_role {
        return Err(Error::AccessDenied);
    }
    Ok(())
}

// #[middleware(Request)]
// async fn log_request(p: &RequestParts) -> Result<(), Error> {
//     Ok(())
// }
//
// #[middleware(Response)]
// async fn log_response(r: &Response<Body>) -> Result<(), Error> {
//     Ok(())
// }
